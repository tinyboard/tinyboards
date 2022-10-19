use crate::{
    models::comment::comment::{Comment, CommentForm},
    traits::Crud,
};
use crate::schema::comment::dsl::*;
use diesel::{
    result::Error,
    PgConnection,
    QueryDsl,
    RunQueryDsl,
};



impl Crud for Comment {
    type Form = CommentForm;
    type IdType = i32;

    fn read(conn: &mut PgConnection, comment_id: i32) -> Result<Self, Error> {
        comment.find(comment_id).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, comment_id: i32) -> Result<usize, Error> {
        diesel::delete(comment.find(comment_id)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &CommentForm) -> Result<Self, Error> {
        let new_comment = diesel::insert_into(comment)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new_comment)
    }
    fn update(conn: &mut PgConnection, comment_id: i32, form: &CommentForm) -> Result<Self, Error> {
        diesel::update(comment.find(comment_id))
            .set(form)
            .get_result::<Self>(conn)
    }
}
