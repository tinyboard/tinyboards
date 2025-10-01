use crate::{
    aggregates::structs::BoardAggregates,
    schema::board_aggregates,
    utils::{get_conn, DbPool},
};
use diesel::{result::Error, *};
use diesel_async::RunQueryDsl;

impl BoardAggregates {
    pub async fn read(pool: &DbPool, board_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        board_aggregates::table
            .filter(board_aggregates::board_id.eq(board_id))
            .first::<Self>(conn)
            .await
    }
}