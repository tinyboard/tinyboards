use crate::{
    models::post::post::{Post, PostForm},
    schema::post,
    traits::Crud,
    utils::naive_now,
};
use diesel::{prelude::*, result::Error, PgConnection};
use tinyboards_utils::TinyBoardsError;

impl Post {
    pub fn submit(conn: &mut PgConnection, form: PostForm) -> Result<Self, TinyBoardsError> {
        Self::create(conn, &form).map_err(|e| TinyBoardsError::from_error_message(e, "could not submit post"))
    }

    pub fn is_post_creator(user_id: i32, post_creator_id: i32) -> bool {
        user_id == post_creator_id
    }

    /// Checks if a post with a given id exists. Don't use if you need a whole Post object.
    pub fn check_if_exists(conn: &mut PgConnection, pid: i32) -> Result<Option<i32>, TinyBoardsError> {
        use crate::schema::post::dsl::*;
        post.select(id)
            .filter(id.eq(pid))
            .first::<i32>(conn)
            .optional()
            .map_err(|e| TinyBoardsError::from_error_message(e, "error while checking existence of post"))
    }

    pub fn update_locked(
        conn: &mut PgConnection,
        post_id: i32,
        new_locked: bool,
    ) -> Result<Self, Error> {
        use crate::schema::post::dsl::*;
        diesel::update(post.find(post_id))
            .set((locked.eq(new_locked), updated.eq(naive_now())))
            .get_result::<Self>(conn)
    }

    pub fn update_deleted(
        conn: &mut PgConnection,
        post_id: i32,
        new_deleted: bool,
    ) -> Result<Self, Error> {
        use crate::schema::post::dsl::*;
        diesel::update(post.find(post_id))
            .set((deleted.eq(new_deleted), updated.eq(naive_now())))
            .get_result::<Self>(conn)
    }
}

impl Crud for Post {
    type Form = PostForm;
    type IdType = i32;

    fn read(conn: &mut PgConnection, post_id: i32) -> Result<Self, Error> {
        post::table.find(post_id).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, post_id: i32) -> Result<usize, Error> {
        diesel::delete(post::table.find(post_id)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &PostForm) -> Result<Self, Error> {
        let new_post = diesel::insert_into(post::table)
            .values(form)
            .get_result::<Self>(conn)?;

        Ok(new_post)
    }
    fn update(conn: &mut PgConnection, post_id: i32, form: &PostForm) -> Result<Self, Error> {
        diesel::update(post::table.find(post_id))
            .set(form)
            .get_result::<Self>(conn)
    }
}
