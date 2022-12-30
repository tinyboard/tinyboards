use crate::{
    models::post::post_votes::{PostVote, PostVoteForm},
    traits::Voteable,
};
use diesel::{prelude::*, PgConnection};
use tinyboards_utils::TinyBoardsError;

impl Voteable for PostVote {
    type Form = PostVoteForm;
    type IdType = i32;

    fn vote(conn: &mut PgConnection, form: &PostVoteForm) -> Result<Self, TinyBoardsError> {
        use crate::schema::post_votes::dsl::*;
        diesel::insert_into(post_votes)
            .values(form)
            .on_conflict((post_id, user_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not create post vote"))
    }

    fn remove(
        conn: &mut PgConnection,
        user_id: i32,
        post_id: i32,
    ) -> Result<usize, TinyBoardsError> {
        use crate::schema::post_votes::dsl;
        diesel::delete(
            dsl::post_votes
                .filter(dsl::post_id.eq(post_id))
                .filter(dsl::user_id.eq(user_id)),
        )
        .execute(conn)
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not remove post vote"))
    }
}
