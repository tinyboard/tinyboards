use crate::{aggregates::structs::PostAggregates, porpl_types::PostId, schema::post_aggregates};
use diesel::{result::Error, *};

impl PostAggregates {
    pub fn read(conn: &mut PgConnection, pid: i32) -> Result<Self, Error> {
        use crate::schema::post_aggregates::dsl::*;
        post_aggregates.filter(post_id.eq(pid)).first::<Self>(conn)
    }
}
