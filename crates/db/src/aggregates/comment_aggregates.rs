use crate::{
    aggregates::structs::CommentAggregates,
    schema::comment_aggregates,
};
use diesel::{result::Error, *};

impl CommentAggregates {
    pub fn read(conn: &mut PgConnection, comment_id: i32) -> Result<Self, Error> {
        comment_aggregates::table
            .filter(comment_aggregates::comment_id.eq(comment_id))
            .first::<Self>(conn)
    }
}