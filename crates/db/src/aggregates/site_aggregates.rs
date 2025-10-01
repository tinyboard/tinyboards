use crate::{aggregates::structs::SiteAggregates, utils::{get_conn, DbPool}};
use diesel::{result::Error, ExpressionMethods, QueryDsl, PgConnection};

impl SiteAggregates {
    pub fn read(conn: &mut PgConnection, sid: i32) -> Result<Self, Error> {
        use crate::schema::site_aggregates::dsl::*;
        use diesel::RunQueryDsl;
        site_aggregates.filter(site_id.eq(sid)).first::<Self>(conn)
    }

    pub async fn read_async(pool: &DbPool, sid: i32) -> Result<Self, Error> {
        use crate::schema::site_aggregates::dsl::*;
        use diesel_async::RunQueryDsl;
        let conn = &mut get_conn(pool).await?;
        site_aggregates.filter(site_id.eq(sid)).first::<Self>(conn).await
    }
}
