use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::site::site::Site as DbSite,
    models::aggregates::SiteAggregates as DbSiteAggregates,
    schema::{site, site_aggregates},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;

use crate::structs::site::{LocalSite, SiteStats};

#[derive(Default)]
pub struct QuerySite;

#[Object]
impl QuerySite {
    /// Get site configuration (public).
    pub async fn site(&self, ctx: &Context<'_>) -> Result<LocalSite> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let db_site: DbSite = site::table
            .first(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to load site: {}", e)))?;

        Ok(LocalSite::from(db_site))
    }

    /// Get site-wide statistics (public).
    pub async fn site_stats(&self, ctx: &Context<'_>) -> Result<SiteStats> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let db_site: DbSite = site::table
            .first(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to load site: {}", e)))?;

        let stats: DbSiteAggregates = site_aggregates::table
            .filter(site_aggregates::site_id.eq(db_site.id))
            .first(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to load site stats: {}", e)))?;

        Ok(SiteStats::from(stats))
    }
}
