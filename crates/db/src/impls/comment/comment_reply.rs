use crate::{
    models::comment::comment_reply::{CommentReply, CommentReplyForm},
    traits::Crud,
};
use diesel::{result::Error, PgConnection, QueryDsl, RunQueryDsl};

impl Crud for CommentReply {
    type Form = CommentReplyForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::comment_reply::dsl::*;
        comment_reply.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::comment_reply::dsl::*;
        diesel::delete(comment_reply.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::comment_reply::dsl::*;
        let new = diesel::insert_into(comment_reply)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::comment_reply::dsl::*;
        diesel::update(comment_reply.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}