/*use crate::aggregates::structs::CommentAggregates;
use crate::schema::comment_aggregates;
use diesel::{result::Error, PgConnection, QueryDsl};

impl CommentAggregates {
    pub fn read(conn: &mut PgConnection, cid: i32) -> Result<Self, Error> {
        comment_aggregates::table
            .filter(comment_aggregates::comment_id.eq(cid))
            .first()
            .load::<Self>(conn)
    }
}*/
