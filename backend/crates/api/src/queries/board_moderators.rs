use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        aggregates::UserAggregates as DbUserAggregates,
        board::board_mods::BoardModerator as DbBoardModerator,
        user::user::User as DbUser,
    },
    schema::{board_moderators, user_aggregates, users},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{helpers::permissions, structs::user::User as GqlUser};

#[derive(Default)]
pub struct QueryBoardModerators;

#[derive(SimpleObject)]
pub struct BoardModerator {
    pub id: ID,
    pub board_id: ID,
    pub user: GqlUser,
    pub created_at: String,
    pub permissions: i32,
    pub rank: i32,
    pub is_invite_accepted: bool,
    pub invite_accepted_at: Option<String>,
}

#[Object]
impl QueryBoardModerators {
    /// Get the list of moderators for a board. This is a public query.
    pub async fn get_board_moderators(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
    ) -> Result<Vec<BoardModerator>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        // Public query — auth is optional.
        let _viewer = permissions::optional_auth(ctx);

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid board ID".to_string()))?;

        let results: Vec<(DbBoardModerator, DbUser)> = board_moderators::table
            .inner_join(users::table.on(board_moderators::user_id.eq(users::id)))
            .filter(board_moderators::board_id.eq(board_uuid))
            .filter(users::deleted_at.is_null())
            .order(board_moderators::rank.asc())
            .select((board_moderators::all_columns, users::all_columns))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Batch load aggregates for all mod users in one query.
        let user_ids: Vec<Uuid> = results.iter().map(|(_, u)| u.id).collect();
        let aggs: Vec<DbUserAggregates> = user_aggregates::table
            .filter(user_aggregates::user_id.eq_any(&user_ids))
            .load(conn)
            .await
            .unwrap_or_default();

        let moderators = results
            .into_iter()
            .map(|(board_mod, user_db)| {
                let agg = aggs.iter().find(|a| a.user_id == user_db.id).cloned();
                BoardModerator {
                    id: ID(board_mod.id.to_string()),
                    board_id: ID(board_mod.board_id.to_string()),
                    user: GqlUser::from_db(user_db, agg),
                    created_at: board_mod.created_at.to_rfc3339(),
                    permissions: board_mod.permissions,
                    rank: board_mod.rank,
                    is_invite_accepted: board_mod.is_invite_accepted,
                    invite_accepted_at: board_mod
                        .invite_accepted_at
                        .map(|d| d.to_rfc3339()),
                }
            })
            .collect();

        Ok(moderators)
    }
}
