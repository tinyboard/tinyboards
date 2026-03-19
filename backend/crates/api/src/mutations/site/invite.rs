use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::config::{SiteInvite as DbSiteInvite, SiteInviteInsertForm},
    schema::site_invites,
    utils::{get_conn, DbPool},
};
use uuid::Uuid;

use crate::helpers::permissions;

#[derive(Default)]
pub struct SiteInvite;

#[Object]
impl SiteInvite {
    /// Create a new invite code (admin only).
    pub async fn create_invite(&self, ctx: &Context<'_>) -> Result<String> {
        let _admin = permissions::require_admin(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let code = Uuid::new_v4().to_string();

        let form = SiteInviteInsertForm {
            verification_code: code.clone(),
        };

        diesel::insert_into(site_invites::table)
            .values(&form)
            .execute(conn)
            .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::Database(e.to_string()))?;

        Ok(code)
    }

    /// Delete an invite code (admin only).
    pub async fn delete_invite(&self, ctx: &Context<'_>, invite_id: ID) -> Result<bool> {
        let _admin = permissions::require_admin(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let id: Uuid = invite_id.parse()
            .map_err(|_| tinyboards_utils::TinyBoardsError::BadRequest("Invalid UUID".to_string()))?;

        let deleted = diesel::delete(site_invites::table.find(id))
            .execute(conn)
            .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::Database(e.to_string()))?;

        Ok(deleted > 0)
    }
}
