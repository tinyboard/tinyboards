use crate::structs::SiteView;
use diesel::{result::Error, *};
use tinyboards_db::{
    aggregates::structs::SiteAggregates,
    schema::{site, site_aggregates},
    models::{site::site::Site,},
};

type SiteViewTuple = (
    Site,
    SiteAggregates,
);

impl SiteView {
    pub fn read_local(
        conn: &mut PgConnection
    ) -> Result<Self, Error> {
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
            .first::<SiteViewTuple>(conn)?;

        
        Ok( SiteView {
            site,
            counts,
        })
            
    }
}