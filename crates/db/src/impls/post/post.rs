use crate::{
    models::{
        post::post::{Post, PostForm},
        post::post_like::{PostLike, PostLikeForm},
    },
    schema::{
        post,
        post_like,
    },
    traits::{
        Crud,
        Likeable,
        Saveable,
    }, 
    utils::naive_now,
};
use diesel::{
    prelude::*,
    result::Error,
    PgConnection,
};
use porpl_utils::PorplError;


impl Post {
    
    pub fn submit(conn: &mut PgConnection, form: PostForm) -> Result<Self, PorplError> {
        Self::create(conn, &form).map_err(|e| {
            eprintln!("ERROR: {}", e);
            PorplError::new(500, String::from("Internal error, please try again later"))
        })
    }

    pub fn is_post_creator(user_id: i32, post_creator_id: i32) -> bool {
        user_id == post_creator_id
    }

    pub fn update_deleted(
        conn: &mut PgConnection,
        post_id: i32,
        new_deleted: bool
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