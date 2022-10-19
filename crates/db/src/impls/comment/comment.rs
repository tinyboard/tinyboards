use crate::schema::comment::dsl::*;
use crate::{
    models::comment::comment::{Comment, CommentForm},
    traits::Crud,
};
use diesel::{prelude::*, result::Error, PgConnection, QueryDsl, RunQueryDsl};
use porpl_utils::PorplError;

impl Comment {
    pub fn submit(conn: &mut PgConnection, form: CommentForm) -> Result<Self, PorplError> {
        Self::create(conn, &form).map_err(|e| {
            eprintln!("ERROR: {}", e);
            PorplError::err_500()
        })
    }
    /// Checks if a comment with a given id exists. Don't use if you need a whole Comment object.
    pub fn check_if_exists(conn: &mut PgConnection, cid: i32) -> Result<Option<i32>, PorplError> {
        use crate::schema::comment::dsl::*;
        comment
            .select(id)
            .filter(id.eq(cid))
            .first::<i32>(conn)
            .optional()
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
    }

    pub fn get_by_id(conn: &mut PgConnection, cid: i32) -> Result<Option<Self>, PorplError> {
        use crate::schema::comment::dsl::*;
        comment
            .filter(id.eq(cid))
            .first::<Self>(conn)
            .optional()
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
    }

    /// Loads list of comments replying to the specified post.
    pub fn replies_to_post(conn: &mut PgConnection, pid: i32) -> Result<Vec<Self>, PorplError> {
        use crate::schema::comment::dsl::*;
        comment
            .filter(post_id.eq(pid))
            .load::<Self>(conn)
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
    }
}

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
