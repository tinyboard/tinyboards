use crate::structs::SiteView;
use diesel::{result::Error, *};
use tinyboards_db::{
    aggregates::structs::SiteAggregates,
    schema::{site, site_aggregates},
    models::{site::site::Site,}, utils::{get_conn, DbPool},
};
use diesel_async::RunQueryDsl;

type SiteViewTuple = (
    Site,
    SiteAggregates,
);

impl SiteView {
    pub async fn read_local(
        pool: &DbPool
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let (
            site, 
            counts
        ) = site::table
            .find(1)
            .inner_join(site_aggregates::table
                .on(site::id.eq(site_aggregates::site_id)),
            )
            .select((
                site::all_columns,
                site_aggregates::all_columns,
            ))
            .first::<SiteViewTuple>(conn)
            .await?;

        
        Ok( SiteView {
            site,
            counts,
        })
            
    }
}