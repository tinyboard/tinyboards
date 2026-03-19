use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::config::SiteInvite as DbSiteInvite,
    schema::site_invites,
    utils::{get_conn, DbPool},
};

use crate::helpers::permissions;

#[derive(Default)]
pub struct QueryInvites;

#[derive(SimpleObject)]
pub struct SiteInviteGql {
    pub id: ID,
    pub verification_code: String,
    pub created_at: String,
}

impl From<DbSiteInvite> for SiteInviteGql {
    fn from(v: DbSiteInvite) -> Self {
        Self {
            id: ID(v.id.to_string()),
            verification_code: v.verification_code,
            created_at: v.created_at.to_rfc3339(),
        }
    }
}

#[Object]
impl QueryInvites {
    /// List site invites (admin only).
    pub async fn list_invites(&self, ctx: &Context<'_>) -> Result<Vec<SiteInviteGql>> {
        let _admin = permissions::require_admin(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let invites: Vec<DbSiteInvite> = site_invites::table
            .order(site_invites::created_at.desc())
            .load(conn)
            .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::Database(e.to_string()))?;

        Ok(invites.into_iter().map(SiteInviteGql::from).collect())
    }
}
