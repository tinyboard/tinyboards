use crate::models::post::post::{Post, PostForm};
use crate::schema::post::dsl::*;
use crate::traits::Crud;
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
    
}

impl Crud for Post {
    type Form = PostForm;
    type IdType = i32;

    fn read(conn: &mut PgConnection, post_id: i32) -> Result<Self, Error> {
        post.find(post_id).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, post_id: i32) -> Result<usize, Error> {
        diesel::delete(post.find(post_id)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &PostForm) -> Result<Self, Error> {
        let new_post = diesel::insert_into(post)
            .values(form)
            .get_result::<Self>(conn)?;

            Ok(new_post)
    }
    fn update(conn: &mut PgConnection, post_id: i32, form: &PostForm) -> Result<Self, Error> {
        diesel::update(post.find(post_id))
            .set(form)
            .get_result::<Self>(conn)
    }
}