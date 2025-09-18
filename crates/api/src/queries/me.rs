use crate::{structs::user::User, LoggedInUser};
use async_graphql::*;
use tinyboards_db::utils::DbPool;
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct MeQuery;

#[Object]
impl MeQuery {
    pub async fn me<'ctx>(&self, ctx: &Context<'ctx>) -> Result<User> {
        let v = ctx.data::<LoggedInUser>()?.require_user()?;

        // Create UserAggregates - we'll need to get this from the database
        let pool = ctx.data::<DbPool>()?;
        use tinyboards_db::aggregates::structs::UserAggregates;

        let user_aggregates = UserAggregates::read(pool, v.id)
            .await
            .unwrap_or_else(|_| UserAggregates {
                id: 0, // Default ID for manually created aggregates
                user_id: v.id,
                post_count: 0,
                post_score: 0,
                comment_count: 0,
                comment_score: 0,
            });

        Ok(User::from((v.clone(), user_aggregates)))
    }

    pub async fn unread_replies_count<'ctx>(&self, ctx: &Context<'ctx>) -> Result<i64> {
        let v = ctx.data::<LoggedInUser>()?.require_user()?;
        let pool = ctx.data::<DbPool>()?;

        // Count unread reply notifications for this user
        use tinyboards_db::schema::notifications;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use tinyboards_db::utils::get_conn;

        let conn = &mut get_conn(pool).await.map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))?;

        let count = notifications::table
            .filter(notifications::recipient_user_id.eq(v.id))
            .filter(notifications::is_read.eq(false))
            .filter(notifications::kind.eq("reply"))
            .count()
            .get_result::<i64>(conn)
            .await
            .map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))?;

        Ok(count)
    }

    pub async fn unread_mentions_count<'ctx>(&self, ctx: &Context<'ctx>) -> Result<i64> {
        let v = ctx.data::<LoggedInUser>()?.require_user()?;
        let pool = ctx.data::<DbPool>()?;

        // Count unread mention notifications for this user
        use tinyboards_db::schema::notifications;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use tinyboards_db::utils::get_conn;

        let conn = &mut get_conn(pool).await.map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))?;

        let count = notifications::table
            .filter(notifications::recipient_user_id.eq(v.id))
            .filter(notifications::is_read.eq(false))
            .filter(notifications::kind.eq("mention"))
            .count()
            .get_result::<i64>(conn)
            .await
            .map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))?;

        Ok(count)
    }
}
