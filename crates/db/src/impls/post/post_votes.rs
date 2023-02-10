use crate::{
    models::post::post_votes::{PostVote, PostVoteForm},
    traits::Voteable, utils::{get_conn, DbPool},
};
use diesel::{prelude::*};
use tinyboards_utils::TinyBoardsError;
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Voteable for PostVote {
    type Form = PostVoteForm;
    type IdType = i32;

    async fn vote(pool: &DbPool, form: &PostVoteForm) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::post_votes::dsl::*;
        diesel::insert_into(post_votes)
            .values(form)
            .on_conflict((post_id, user_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not create post vote"))
    }

    async fn remove(
        pool: &DbPool,
        user_id: i32,
        post_id: i32,
    ) -> Result<usize, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::post_votes::dsl;
        diesel::delete(
            dsl::post_votes
                .filter(dsl::post_id.eq(post_id))
                .filter(dsl::user_id.eq(user_id)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not remove post vote"))
    }
}
