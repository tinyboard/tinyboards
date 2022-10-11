use crate::{
    aggregates::structs::BoardAggregates,
    schema::board_aggregates,
};
use diesel::{result::Error, *};

impl BoardAggregates {
    pub fn read(conn: &mut PgConnection, board_id: i32) -> Result<Self, Error> {
        board_aggregates::table
            .filter(board_aggregates::board_id.eq(board_id))
            .first::<Self>(conn)
    }
}