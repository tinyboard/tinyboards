use crate::{aggregates::structs::SiteAggregates};
use diesel::{result::Error, *};

impl SiteAggregates {
    pub fn read(conn: &mut PgConnection, sid: i32) -> Result<Self, Error> {
        use crate::schema::site_aggregates::dsl::*;
        site_aggregates.filter(site_id.eq(sid)).first::<Self>(conn)
    }
}
