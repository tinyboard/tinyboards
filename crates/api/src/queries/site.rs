use async_graphql::*;
use tinyboards_db::{
    aggregates::structs::SiteAggregates,
    models::site::site::Site as DbSite,
    utils::DbPool
};
use tinyboards_utils::TinyBoardsError;

use crate::structs::site::{LocalSite, SiteStats};

#[derive(Default)]
pub struct QuerySite;

#[Object]
impl QuerySite {
    pub async fn site(&self, ctx: &Context<'_>) -> Result<LocalSite> {
        let pool = ctx.data_unchecked::<DbPool>();

        DbSite::read(pool)
            .await
            .map(LocalSite::from)
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load site, sorry :(").into()
            })
    }

    pub async fn site_stats(&self, ctx: &Context<'_>) -> Result<SiteStats> {
        let pool = ctx.data_unchecked::<DbPool>();

        let site = DbSite::read(pool).await.map_err(|e| -> Error {
            TinyBoardsError::from_error_message(e, 500, "Failed to load site").into()
        })?;

        SiteAggregates::read_async(pool, site.id)
            .await
            .map(SiteStats::from)
            .map_err(|e| -> Error {
                TinyBoardsError::from_error_message(e, 500, "Failed to load site statistics").into()
            })
    }
}
