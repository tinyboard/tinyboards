use crate::{
    models::comment::comment_votes::{CommentVote, CommentVoteForm},
    traits::Voteable, 
    utils::{get_conn, DbPool},
};
use diesel::{prelude::*};
use tinyboards_utils::TinyBoardsError;
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Voteable for CommentVote {
    type Form = CommentVoteForm;
    type IdType = i32;

    async fn vote(pool: &DbPool, form: &CommentVoteForm) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comment_votes::dsl::*;
        diesel::insert_into(comment_votes)
            .values(form)
            .on_conflict((comment_id, person_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not vote on comment"))
    }

    async fn remove(pool: &DbPool, person_id: i32, cid: i32) -> Result<usize, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comment_votes::dsl;
        diesel::delete(
            dsl::comment_votes
                .filter(dsl::comment_id.eq(cid))
                .filter(dsl::person_id.eq(person_id)),
        )
        .execute(conn)
        .await
        .map_err(|e| {
            TinyBoardsError::from_error_message(e, 500, "could not remove vote on comment")
        })
    }
}
