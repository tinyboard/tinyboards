use crate::{
    models::post::post_votes::{PostVote, PostVoteForm},
    traits::Voteable,
    utils::{get_conn, DbPool},
};
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

impl PostVote {
    /// Returns my vote type for each post ids.
    pub async fn get_my_vote_for_ids(
        pool: &DbPool,
        ids: Vec<i32>,
        for_user_id: i32,
    ) -> Result<Vec<(i32, i32)>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::{post_votes, posts};

        posts::table
            .left_join(
                post_votes::table.on(post_votes::post_id
                    .eq(posts::id)
                    .and(post_votes::user_id.eq(for_user_id))),
            )
            .filter(posts::id.eq_any(ids))
            .select((posts::id, post_votes::score.nullable()))
            .load::<(i32, Option<i32>)>(conn)
            .await
            .map(|list| {
                list.into_iter()
                    .map(|(post_id, vote_type)| (post_id, vote_type.unwrap_or(0)))
                    .collect::<Vec<(i32, i32)>>()
            })
    }
}

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

    async fn remove(pool: &DbPool, user_id: i32, post_id: i32) -> Result<usize, TinyBoardsError> {
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
