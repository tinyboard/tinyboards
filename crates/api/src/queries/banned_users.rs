use async_graphql::*;
use tinyboards_db::{
    aggregates::structs::UserAggregates,
    models::user::user::{AdminPerms, User},
    utils::{DbPool, get_conn},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

use crate::{
    structs::user::User as GqlUser,
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
    ) -> Result<Vec<GqlUser>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        // Only admins can view banned users
        if user.admin_level < AdminPerms::Users as i32 {
            return Err(TinyBoardsError::from_message(
                403,
                "Only admins can view banned users",
            )
            .into());
        }

        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(25).min(100); // Cap at 100
        let offset = (page - 1) * limit;

        use tinyboards_db::schema::users;

        let banned_users = users::table
            .filter(users::is_banned.eq(true))
            .order(users::creation_date.desc())
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<User>(conn)
            .await?;

        let mut result = Vec::new();
        for user_db in banned_users {
            // Create default aggregates for banned users
            let aggregates = UserAggregates {
                id: 0, // Default ID for manually created aggregates
                user_id: user_db.id,
                post_count: 0,
                post_score: 0,
                comment_count: 0,
                comment_score: 0,
                rep: 0,
            };

            result.push(GqlUser::from((user_db, aggregates)));
        }

        Ok(result)
    }
}