use async_graphql::*;
use tinyboards_db::{
    aggregates::structs::UserAggregates,
    models::{
        board::{
            board_mods::BoardModerator,
            board_user_bans::BoardUserBan,
            boards::Board,
        },
        user::user::{AdminPerms, User},
    },
    traits::Crud,
    utils::{DbPool, get_conn},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

use crate::{
    structs::{boards::Board as GqlBoard, user::User as GqlUser},
    LoggedInUser,
};

#[derive(Default)]
pub struct QueryBoardManagement;

#[derive(SimpleObject)]
pub struct BoardSettings {
    pub board: GqlBoard,
    pub is_owner: bool,
    pub moderator_permissions: Option<i32>,
}

#[derive(SimpleObject)]
pub struct BannedUser {
    pub id: i32,
    pub user: GqlUser,
    pub board_id: i32,
    pub ban_date: String,
    pub expires: Option<String>,
}

#[Object]
impl QueryBoardManagement {
    /// Get detailed board settings (moderators only)
    pub async fn get_board_settings(
        &self,
        ctx: &Context<'_>,
        board_id: i32,
    ) -> Result<BoardSettings> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Check if board exists
        let board = Board::read(pool, board_id).await
            .map_err(|_| TinyBoardsError::from_message(404, "Board not found"))?;

        // Check if user has permission to view board settings
        let is_owner = match BoardModerator::get_by_user_id_for_board(pool, user.id, board_id, true).await {
            Ok(mod_entry) => mod_entry.rank == 0, // Rank 0 is typically the owner
            Err(_) => false,
        };
        let is_admin = user.has_permission(AdminPerms::Users);

        let moderator_permissions = if !is_admin && !is_owner {
            // Check if user is a moderator
            match BoardModerator::get_by_user_id_for_board(pool, user.id, board_id, true).await {
                Ok(mod_entry) => Some(mod_entry.permissions),
                Err(_) => {
                    return Err(TinyBoardsError::from_message(
                        403,
                        "You must be a moderator, owner, or admin to view board settings",
                    )
                    .into());
                }
            }
        } else {
            // Admins and owners have full permissions
            Some(64) // Full permissions
        };

        // Convert board to GraphQL format
        let board_with_counts = Board::get_with_counts_for_ids(pool, vec![board_id]).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to get board with counts"))?
            .into_iter()
            .next()
            .ok_or_else(|| TinyBoardsError::from_message(500, "Failed to get board with counts"))?;

        let gql_board = GqlBoard::from(board_with_counts);

        Ok(BoardSettings {
            board: gql_board,
            is_owner,
            moderator_permissions,
        })
    }

    /// Get list of users banned from a board (moderators only)
    pub async fn get_board_banned_users(
        &self,
        ctx: &Context<'_>,
        board_id: i32,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<BannedUser>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Check if board exists
        let board = Board::read(pool, board_id).await
            .map_err(|_| TinyBoardsError::from_message(404, "Board not found"))?;

        // Check if user has permission to view banned users
        let can_view_bans = if user.has_permission(AdminPerms::Users) {
            true // Admins can view bans on any board
        } else if match BoardModerator::get_by_user_id_for_board(pool, user.id, board_id, true).await {
            Ok(mod_entry) => mod_entry.rank == 0, // Board owner
            Err(_) => false,
        } {
            true // Board owners can view bans
        } else {
            // Check if user is a moderator with Users permission
            match BoardModerator::get_by_user_id_for_board(pool, user.id, board_id, true).await {
                Ok(mod_entry) => {
                    // Check if user has Users (16) or Full (64) permissions
                    let permissions = mod_entry.permissions;
                    permissions & (16 | 64) != 0
                }
                Err(_) => false,
            }
        };

        if !can_view_bans {
            return Err(TinyBoardsError::from_message(
                403,
                "You don't have permission to view banned users for this board",
            )
            .into());
        }

        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(25).min(100); // Cap at 100
        let offset = (page - 1) * limit;

        use tinyboards_db::schema::{board_user_bans, users};

        let results = board_user_bans::table
            .inner_join(users::table.on(board_user_bans::user_id.eq(users::id)))
            .filter(board_user_bans::board_id.eq(board_id))
            .order(board_user_bans::creation_date.desc())
            .limit(limit as i64)
            .offset(offset as i64)
            .select((board_user_bans::all_columns, users::all_columns))
            .load::<(BoardUserBan, User)>(conn)
            .await?;

        let mut banned_users = Vec::new();
        for (ban, user_db) in results {
            // Create default aggregates for banned users
            let aggregates = UserAggregates {
                id: 0, // Default ID for manually created aggregates
                user_id: user_db.id,
                post_count: 0,
                post_score: 0,
                comment_count: 0,
                comment_score: 0,
            };

            banned_users.push(BannedUser {
                id: ban.id,
                user: GqlUser::from((user_db, aggregates)),
                board_id: ban.board_id,
                ban_date: ban.creation_date.to_string(),
                expires: ban.expires.map(|d| d.to_string()),
            });
        }

        Ok(banned_users)
    }
}