use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        aggregates::UserAggregates as DbUserAggregates,
        user::user::{AdminPerms, User as DbUser},
    },
    schema::{user_aggregates, users},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{helpers::permissions, structs::user::User as GqlUser};

#[derive(Default)]
pub struct QueryBannedUsers;

#[derive(SimpleObject)]
pub struct BannedUsersResponse {
    pub users: Vec<GqlUser>,
    pub total_count: i64,
}

#[Object]
impl QueryBannedUsers {
    /// List site-wide banned users. Admin-only (requires Users permission).
    pub async fn list_banned_users(
        &self,
        ctx: &Context<'_>,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<BannedUsersResponse> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        // Require admin with Users permission.
        permissions::require_admin_permission(ctx, AdminPerms::Users)
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let limit = limit.unwrap_or(25).min(100);
        let offset = (page.unwrap_or(1) - 1) * limit;

        let banned_users: Vec<DbUser> = users::table
            .filter(users::is_banned.eq(true))
            .filter(users::deleted_at.is_null())
            .order(users::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Batch load aggregates.
        let user_ids: Vec<Uuid> = banned_users.iter().map(|u| u.id).collect();
        let aggs: Vec<DbUserAggregates> = user_aggregates::table
            .filter(user_aggregates::user_id.eq_any(&user_ids))
            .load(conn)
            .await
            .unwrap_or_default();

        let users: Vec<GqlUser> = banned_users
            .into_iter()
            .map(|u| {
                let agg = aggs.iter().find(|a| a.user_id == u.id).cloned();
                GqlUser::from_db(u, agg)
            })
            .collect();

        // Count total banned users for pagination metadata.
        let total_count: i64 = users::table
            .filter(users::is_banned.eq(true))
            .filter(users::deleted_at.is_null())
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(BannedUsersResponse { users, total_count })
    }
}
