use async_graphql::*;
use tinyboards_db::{
    aggregates::structs::PersonAggregates,
    models::{
        board::board_mods::BoardModerator as DbBoardModerator,
        person::{local_user::LocalUser, person::Person, user::User},
    },
    utils::{DbPool, get_conn},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

use crate::{
    structs::person::Person as GqlPerson,
    LoggedInUser,
};

#[derive(Default)]
pub struct QueryBoardModerators;

#[derive(SimpleObject)]
pub struct BoardModerator {
    pub id: i32,
    pub board_id: i32,
    pub person: GqlPerson,
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

        use tinyboards_db::schema::{board_mods, person};

        let results = board_mods::table
            .inner_join(person::table.on(board_mods::person_id.eq(person::id)))
            .filter(board_mods::board_id.eq(board_id))
            .order(board_mods::rank.asc())
            .select((board_mods::all_columns, person::all_columns))
            .load::<(DbBoardModerator, Person)>(conn)
            .await?;

        let mut result = Vec::new();
        for (board_mod, person) in results {
            // Create default aggregates for moderators
            let aggregates = PersonAggregates {
                id: person.id,
                person_id: person.id,
                post_count: 0,
                post_score: 0,
                comment_count: 0,
                comment_score: 0,
                rep: 0,
            };

            // Get local user if exists (optional)
            use tinyboards_db::schema::local_user;
            let local_user = local_user::table
                .filter(local_user::person_id.eq(person.id))
                .first::<LocalUser>(conn)
                .await
                .optional()?;

            let user = User {
                person,
                counts: aggregates,
                local_user,
            };

            result.push(BoardModerator {
                id: board_mod.id,
                board_id: board_mod.board_id,
                person: GqlPerson::from(user),
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