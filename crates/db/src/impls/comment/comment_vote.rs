use crate::{
    models::comment::comment_vote::{CommentVote, CommentVoteForm},
    traits::Voteable,
};
use diesel::{prelude::*, PgConnection};
use porpl_utils::PorplError;

impl Voteable for CommentVote {
    type Form = CommentVoteForm;
    type IdType = i32;

    fn vote(conn: &mut PgConnection, form: &CommentVoteForm) -> Result<Self, PorplError> {
        use crate::schema::comment_vote::dsl::*;
        diesel::insert_into(comment_vote)
            .values(form)
            .on_conflict((comment_id, user_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
    }

    fn remove(conn: &mut PgConnection, user_id: i32, cid: i32) -> Result<usize, PorplError> {
        use crate::schema::comment_vote::dsl;
        diesel::delete(
            dsl::comment_vote
                .filter(dsl::comment_id.eq(cid))
                .filter(dsl::user_id.eq(user_id)),
        )
        .execute(conn)
        .map_err(|e| {
            eprintln!("ERROR: {}", e);
            PorplError::err_500()
        })
    }
}
