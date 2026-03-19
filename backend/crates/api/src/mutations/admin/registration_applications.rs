use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::config::RegistrationApplication as DbRegApp,
    models::user::user::AdminPerms,
    schema::{registration_applications, users},
    utils::{get_conn, DbPool},
};
use uuid::Uuid;

use crate::helpers::permissions;

#[derive(Default)]
pub struct RegistrationApplicationMutations;

#[Object]
impl RegistrationApplicationMutations {
    /// Approve a registration application (admin with Users permission).
    pub async fn approve_application(&self, ctx: &Context<'_>, application_id: ID) -> Result<bool> {
        let admin = permissions::require_admin_permission(ctx, AdminPerms::Users)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let id: Uuid = application_id.parse()
            .map_err(|_| tinyboards_utils::TinyBoardsError::BadRequest("Invalid UUID".to_string()))?;

        let app: DbRegApp = registration_applications::table.find(id)
            .first(conn)
            .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::NotFound(format!("Application not found: {}", e)))?;

        // Update the application with the approving admin
        diesel::update(registration_applications::table.find(id))
            .set(registration_applications::admin_id.eq(Some(admin.id)))
            .execute(conn)
            .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::Database(e.to_string()))?;

        // Mark the user as application accepted
        diesel::update(users::table.find(app.user_id))
            .set(users::is_application_accepted.eq(true))
            .execute(conn)
            .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::Database(e.to_string()))?;

        Ok(true)
    }

    /// Deny a registration application with reason (admin with Users permission).
    pub async fn deny_application(
        &self,
        ctx: &Context<'_>,
        application_id: ID,
        reason: Option<String>,
    ) -> Result<bool> {
        let admin = permissions::require_admin_permission(ctx, AdminPerms::Users)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let id: Uuid = application_id.parse()
            .map_err(|_| tinyboards_utils::TinyBoardsError::BadRequest("Invalid UUID".to_string()))?;

        // Verify the application exists
        let _app: DbRegApp = registration_applications::table.find(id)
            .first(conn)
            .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::NotFound(format!("Application not found: {}", e)))?;

        diesel::update(registration_applications::table.find(id))
            .set((
                registration_applications::admin_id.eq(Some(admin.id)),
                registration_applications::deny_reason.eq(reason),
            ))
            .execute(conn)
            .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::Database(e.to_string()))?;

        Ok(true)
    }
}
