use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::config::RegistrationApplication as DbRegApp,
    models::user::user::AdminPerms,
    schema::registration_applications,
    utils::{get_conn, DbPool},
};

use crate::helpers::permissions;

#[derive(Default)]
pub struct RegistrationApplicationQueries;

#[derive(SimpleObject)]
pub struct RegistrationApplication {
    pub id: ID,
    pub user_id: ID,
    pub answer: String,
    pub admin_id: Option<ID>,
    pub deny_reason: Option<String>,
    pub created_at: String,
}

impl From<DbRegApp> for RegistrationApplication {
    fn from(v: DbRegApp) -> Self {
        Self {
            id: ID(v.id.to_string()),
            user_id: ID(v.user_id.to_string()),
            answer: v.answer,
            admin_id: v.admin_id.map(|id| ID(id.to_string())),
            deny_reason: v.deny_reason,
            created_at: v.created_at.to_rfc3339(),
        }
    }
}

#[Object]
impl RegistrationApplicationQueries {
    /// List registration applications (admin with Users permission).
    pub async fn list_registration_applications(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<RegistrationApplication>> {
        let _admin = permissions::require_admin_permission(ctx, AdminPerms::Users)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let limit = limit.unwrap_or(20).min(50);
        let offset = offset.unwrap_or(0);

        let apps: Vec<DbRegApp> = registration_applications::table
            .order(registration_applications::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::Database(e.to_string()))?;

        Ok(apps.into_iter().map(RegistrationApplication::from).collect())
    }

    /// Get a specific registration application by ID.
    pub async fn get_registration_application(
        &self,
        ctx: &Context<'_>,
        application_id: ID,
    ) -> Result<RegistrationApplication> {
        let _admin = permissions::require_admin_permission(ctx, AdminPerms::Users)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let id: uuid::Uuid = application_id.parse()
            .map_err(|_| tinyboards_utils::TinyBoardsError::BadRequest("Invalid UUID".to_string()))?;

        let app: DbRegApp = registration_applications::table
            .find(id)
            .first(conn)
            .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::NotFound(format!("Application not found: {}", e)))?;

        Ok(RegistrationApplication::from(app))
    }

    /// Count of pending registration applications.
    pub async fn registration_applications_count(&self, ctx: &Context<'_>) -> Result<i64> {
        let _admin = permissions::require_admin_permission(ctx, AdminPerms::Users)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        // Applications without an admin_id are pending
        let count: i64 = registration_applications::table
            .filter(registration_applications::admin_id.is_null())
            .count()
            .get_result(conn)
            .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::Database(e.to_string()))?;

        Ok(count)
    }
}
