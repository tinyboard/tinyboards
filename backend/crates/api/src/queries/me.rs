use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::aggregates::UserAggregates as DbUserAggregates,
    schema::{notifications, user_aggregates},
    utils::{get_conn, DbPool},
};

use crate::{helpers::permissions, structs::user::User};

#[derive(Default)]
pub struct MeQuery;

#[derive(SimpleObject)]
pub struct MeResponse {
    pub user: User,
    pub unread_notifications_count: i64,
}

#[Object]
impl MeQuery {
    /// Get current authenticated user with unread counts.
    pub async fn me(&self, ctx: &Context<'_>) -> Result<MeResponse> {
        let user = permissions::require_auth(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let agg: Option<DbUserAggregates> = user_aggregates::table
            .filter(user_aggregates::user_id.eq(user.id))
            .first(conn)
            .await
            .optional()
            .map_err(|e| tinyboards_utils::TinyBoardsError::Database(e.to_string()))?;

        let unread: i64 = notifications::table
            .filter(notifications::recipient_user_id.eq(user.id))
            .filter(notifications::is_read.eq(false))
            .count()
            .get_result(conn)
            .await
            .unwrap_or(0);

        Ok(MeResponse {
            user: User::from_db(user.clone(), agg),
            unread_notifications_count: unread,
        })
    }
}
