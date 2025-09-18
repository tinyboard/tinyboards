use async_graphql::*;
use tinyboards_db::{
    aggregates::structs::UserAggregates,
    models::{
        board::board_mods::BoardModerator as DbBoardModerator,
        user::user::User,
    },
    utils::{DbPool, get_conn},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{
    structs::user::User as GqlUser,
    LoggedInUser,
};

#[derive(Default)]
pub struct QueryBoardModerators;

#[derive(SimpleObject)]
pub struct BoardModerator {
    pub id: i32,
    pub board_id: i32,
    pub user: GqlUser,
    pub creation_date: String,
    pub permissions: i32,
    pub rank: i32,
    pub invite_accepted: bool,
    pub invite_accepted_date: Option<String>,
}

#[Object]
impl QueryBoardModerators {
    /// Get board moderators
    pub async fn get_board_moderators(
        &self,
        ctx: &Context<'_>,
        board_id: i32,
    ) -> Result<Vec<BoardModerator>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let _user = ctx.data_unchecked::<LoggedInUser>(); // Allow unauthenticated access

        use tinyboards_db::schema::{board_mods, users};

        let results = board_mods::table
            .inner_join(users::table.on(board_mods::user_id.eq(users::id)))
            .filter(board_mods::board_id.eq(board_id))
            .order(board_mods::rank.asc())
            .select((board_mods::all_columns, users::all_columns))
            .load::<(DbBoardModerator, User)>(conn)
            .await?;

        let mut result = Vec::new();
        for (board_mod, user_db) in results {
            // Create default aggregates for moderators
            let aggregates = UserAggregates {
                id: 0, // Default ID for manually created aggregates
                user_id: user_db.id,
                post_count: 0,
                post_score: 0,
                comment_count: 0,
                comment_score: 0,
            };

            result.push(BoardModerator {
                id: board_mod.id,
                board_id: board_mod.board_id,
                user: GqlUser::from((user_db, aggregates)),
                creation_date: board_mod.creation_date.to_string(),
                permissions: board_mod.permissions,
                rank: board_mod.rank,
                invite_accepted: board_mod.invite_accepted,
                invite_accepted_date: board_mod.invite_accepted_date.map(|d| d.to_string()),
            });
        }

        Ok(result)
    }
}