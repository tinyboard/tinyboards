use crate::structs::SiteView;
use diesel::{result::Error, *};
use tinyboards_db::{
    aggregates::structs::SiteAggregates,
    schema::{site, local_site, local_site_rate_limit, site_aggregates},
    models::{site::{site::Site, local_site_rate_limit::LocalSiteRateLimit, local_site::LocalSite},}, utils::{get_conn, DbPool},
};
use diesel_async::RunQueryDsl;

type SiteViewTuple = (
    Site,
    LocalSite,
    LocalSiteRateLimit,
    SiteAggregates,
);

impl SiteView {
    pub async fn read_local(
        pool: &DbPool
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let (
            mut site,
            local_site,
            local_site_rate_limit, 
            counts
        ) = site::table
            .inner_join(local_site::table)
            .inner_join(
                local_site_rate_limit::table.on(local_site::id.eq(local_site_rate_limit::local_site_id)),
            )
            .inner_join(site_aggregates::table)
            .select((
                site::all_columns,
                local_site::all_columns,
                local_site_rate_limit::all_columns,
                site_aggregates::all_columns,
            ))
            .first::<SiteViewTuple>(conn)
            .await?;

        site.private_key = None;
        Ok( SiteView {
            site,
            local_site,
            local_site_rate_limit,
            counts,
        })
            
    }
}