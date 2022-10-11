use crate::aggregates::structs::CommentAggregates;
use diesel::{result::Error, *};

impl CommentAggregates {
    pub fn read(conn: &mut PgConnection, comment_id: i32) -> Result<Self, Error> {
        use crate::schema::comment_aggregates::dsl::*;
        comment_aggregates
            .filter(comment_id.eq(comment_id))
            .first::<Self>(conn)
    }
}
