use crate::{
    models::{
        post::post_like::{PostLike, PostLikeForm},
    },
    traits::{
        Likeable,
    },
};
use diesel::{
    prelude::*,
    PgConnection,
};
use porpl_utils::PorplError;

impl Likeable for PostLike {
    type Form = PostLikeForm;
    type IdType = i32;

    fn vote(conn: &mut PgConnection, form: &PostLikeForm) -> Result<Self, PorplError> {
        use crate::schema::post_like::dsl::*;
        diesel::insert_into(post_like)
            .values(form)
            .on_conflict((post_id, user_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
        })
    }

    fn remove(conn: &mut PgConnection, user_id: i32, post_id: i32) -> Result<usize, PorplError> {
        use crate::schema::post_like::dsl;
        diesel::delete(
            dsl::post_like
                .filter(dsl::post_id.eq(post_id))
                .filter(dsl::user_id.eq(user_id)),
        )
        .execute(conn)
        .map_err(|e| {
            eprintln!("ERROR: {}", e);
            PorplError::err_500()
            }
        )
    }
}