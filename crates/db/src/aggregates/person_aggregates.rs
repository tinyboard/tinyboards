use crate::{
    aggregates::structs::PersonAggregates,
    schema::person_aggregates,
};
use diesel::{result::Error, *};

impl PersonAggregates {
    pub fn read(conn: &mut PgConnection, person_id: i32) -> Result<Self, Error> {
        person_aggregates::table
            .filter(person_aggregates::person_id.eq(person_id))
            .first::<Self>(conn)
    }
}