use async_graphql::*;
use tinyboards_db::{
    aggregates::structs::PersonAggregates,
    models::{
        person::{local_user::{AdminPerms, LocalUser}, person::Person, user::User},
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
pub struct QueryBannedUsers;

#[Object]
impl QueryBannedUsers {
    /// List banned users (admin only)
    pub async fn list_banned_users(
        &self,
        ctx: &Context<'_>,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<GqlPerson>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        // Only admins can view banned users
        if !user.has_permission(AdminPerms::Users) {
            return Err(TinyBoardsError::from_message(
                403,
                "Only admins can view banned users",
            )
            .into());
        }

        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(25).min(100); // Cap at 100
        let offset = (page - 1) * limit;

        use tinyboards_db::schema::person;

        let banned_users = person::table
            .filter(person::is_banned.eq(true))
            .order(person::creation_date.desc())
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<Person>(conn)
            .await?;

        let mut result = Vec::new();
        for person in banned_users {
            // Create default aggregates for banned users
            let aggregates = PersonAggregates {
                id: person.id,
                person_id: person.id,
                post_count: 0,
                post_score: 0,
                comment_count: 0,
                comment_score: 0,
                rep: 0,
            };

            // Get local user if exists (optional for banned users)
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

            result.push(GqlPerson::from(user));
        }

        Ok(result)
    }
}