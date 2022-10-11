use crate::{
    aggregates::structs::UserAggregates,
    schema::user_aggregates,
};
use diesel::{result::Error, *};

impl UserAggregates {
    pub fn read(conn: &mut PgConnection, user_id: i32) -> Result<Self, Error> {
        user_aggregates::table
            .filter(user_aggregates::user_id.eq(user_id))
            .first::<Self>(conn)
    }
}