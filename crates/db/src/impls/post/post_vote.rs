use crate::{
    models::{
        post::post_vote::{PostVote, PostVoteForm},
    },
    traits::Voteable,
};
use diesel::{
    prelude::*,
    PgConnection,
};
use porpl_utils::PorplError;

impl Voteable for PostVote {
    type Form = PostVoteForm;
    type IdType = i32;

    fn vote(conn: &mut PgConnection, form: &PostVoteForm) -> Result<Self, PorplError> {
        use crate::schema::post_vote::dsl::*;
        diesel::insert_into(post_vote)
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
        use crate::schema::post_vote::dsl;
        diesel::delete(
            dsl::post_vote
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