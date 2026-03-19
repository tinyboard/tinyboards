use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        aggregates::{BoardAggregates as DbBoardAggregates, UserAggregates as DbUserAggregates},
        board::{
            board_mods::{BoardModerator as DbBoardModerator, ModPerms},
            boards::Board as DbBoard,
        },
        social::BoardUserBan,
        user::user::{AdminPerms, User as DbUser},
    },
    schema::{board_aggregates, board_moderators, board_user_bans, boards, user_aggregates, users},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{
    helpers::permissions,
    structs::{boards::Board as GqlBoard, user::User as GqlUser},
};

#[derive(Default)]
pub struct QueryBoardManagement;

#[derive(SimpleObject)]
pub struct BoardSettings {
    pub board: GqlBoard,
    pub is_owner: bool,
    /// The caller's effective moderator permission bitmask for this board.
    /// `None` if the caller has no mod row (admin path bypasses this).
    pub moderator_permissions: Option<i32>,
}

#[derive(SimpleObject)]
pub struct BoardBannedUser {
    pub id: ID,
    pub user: GqlUser,
    pub board_id: ID,
    pub ban_date: String,
    pub expires: Option<String>,
}

#[Object]
impl QueryBoardManagement {
    /// Get detailed board settings. Requires the caller to be a board
    /// moderator with Config permission, or a site admin with Boards permission.
    pub async fn get_board_settings(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
    ) -> Result<BoardSettings> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid board ID".to_string()))?;

        // Verify caller has the Config mod permission or admin Boards fallback.
        let caller = permissions::require_board_mod_or_admin(
            ctx,
            pool,
            board_uuid,
            ModPerms::Config,
            Some(AdminPerms::Boards),
        )
        .await
        .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        // Load the board.
        let db_board: DbBoard = boards::table
            .filter(boards::id.eq(board_uuid))
            .filter(boards::deleted_at.is_null())
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found".to_string()))?;

        // Load aggregates.
        let agg: Option<DbBoardAggregates> = board_aggregates::table
            .filter(board_aggregates::board_id.eq(board_uuid))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let gql_board = GqlBoard::from_db(db_board, agg);

        // Check caller's mod entry so we can surface their permissions and
        // determine whether they are the board owner (rank 0).
        let mod_row: Option<DbBoardModerator> = board_moderators::table
            .filter(board_moderators::board_id.eq(board_uuid))
            .filter(board_moderators::user_id.eq(caller.id))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let is_owner = mod_row.as_ref().map(|m| m.rank == 0).unwrap_or(false);
        let moderator_permissions = mod_row.map(|m| m.permissions);

        Ok(BoardSettings {
            board: gql_board,
            is_owner,
            moderator_permissions,
        })
    }

    /// Get the list of users banned from a board. Requires the caller to be
    /// a board moderator with Users permission, or a site admin with Boards
    /// permission.
    pub async fn get_board_banned_users(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<BoardBannedUser>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid board ID".to_string()))?;

        // Require mod-level Users permission or admin Boards fallback.
        permissions::require_board_mod_or_admin(
            ctx,
            pool,
            board_uuid,
            ModPerms::Users,
            Some(AdminPerms::Boards),
        )
        .await
        .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let limit = limit.unwrap_or(25).min(100);
        let offset = (page.unwrap_or(1) - 1) * limit;

        let results: Vec<(BoardUserBan, DbUser)> = board_user_bans::table
            .inner_join(users::table.on(board_user_bans::user_id.eq(users::id)))
            .filter(board_user_bans::board_id.eq(board_uuid))
            .filter(users::deleted_at.is_null())
            .order(board_user_bans::created_at.desc())
            .limit(limit)
            .offset(offset)
            .select((board_user_bans::all_columns, users::all_columns))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Batch load user aggregates.
        let user_ids: Vec<Uuid> = results.iter().map(|(_, u)| u.id).collect();
        let aggs: Vec<DbUserAggregates> = user_aggregates::table
            .filter(user_aggregates::user_id.eq_any(&user_ids))
            .load(conn)
            .await
            .unwrap_or_default();

        let banned_users = results
            .into_iter()
            .map(|(ban, user_db)| {
                let agg = aggs.iter().find(|a| a.user_id == user_db.id).cloned();
                BoardBannedUser {
                    id: ID(ban.id.to_string()),
                    user: GqlUser::from_db(user_db, agg),
                    board_id: ID(ban.board_id.to_string()),
                    ban_date: ban.created_at.to_rfc3339(),
                    expires: ban.expires_at.map(|d| d.to_rfc3339()),
                }
            })
            .collect();

        Ok(banned_users)
    }

    /// Get the list of boards that the authenticated user moderates.
    pub async fn get_moderated_boards(
        &self,
        ctx: &Context<'_>,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<GqlBoard>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let caller = permissions::require_auth_not_banned(ctx)
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let limit = limit.unwrap_or(25).min(100);
        let offset = (page.unwrap_or(1) - 1) * limit;

        // Fetch boards the caller moderates, joined with aggregates.
        let results: Vec<(DbBoard, Option<DbBoardAggregates>)> = boards::table
            .inner_join(
                board_moderators::table
                    .on(board_moderators::board_id.eq(boards::id)),
            )
            .left_join(
                board_aggregates::table.on(board_aggregates::board_id.eq(boards::id)),
            )
            .filter(board_moderators::user_id.eq(caller.id))
            .filter(boards::deleted_at.is_null())
            .order(board_moderators::rank.asc())
            .limit(limit)
            .offset(offset)
            .select((boards::all_columns, board_aggregates::all_columns.nullable()))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let boards_out = results
            .into_iter()
            .map(|(b, agg)| GqlBoard::from_db(b, agg))
            .collect();

        Ok(boards_out)
    }
}
