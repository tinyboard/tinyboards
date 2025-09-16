use crate::{
    models::comment::comment_votes::{CommentVote, CommentVoteForm},
    traits::Voteable,
    utils::{get_conn, DbPool},
};
use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

impl CommentVote {
    /// Returns my vote type for each comment ids.
    pub async fn get_my_vote_for_ids(
        pool: &DbPool,
        ids: Vec<i32>,
        for_user_id: i32,
    ) -> Result<Vec<(i32, i16)>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::{comment_votes, comments};

        comments::table
            .left_join(
                comment_votes::table.on(comment_votes::comment_id
                    .eq(comments::id)
                    .and(comment_votes::user_id.eq(for_user_id))),
            )
            .filter(comments::id.eq_any(ids))
            .select((comments::id, comment_votes::score.nullable()))
            .load::<(i32, Option<i16>)>(conn)
            .await
            .map(|list| {
                list.into_iter()
                    .map(|(comment_id, vote_type)| (comment_id, vote_type.unwrap_or(0)))
                    .collect::<Vec<(i32, i16)>>()
            })
    }
}

#[async_trait::async_trait]
impl Voteable for CommentVote {
    type Form = CommentVoteForm;
    type IdType = i32;

    async fn vote(pool: &DbPool, form: &CommentVoteForm) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comment_votes::dsl::*;
        diesel::insert_into(comment_votes)
            .values(form)
            .on_conflict((comment_id, user_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not vote on comment"))
    }

    async fn remove(pool: &DbPool, user_id: i32, cid: i32) -> Result<usize, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comment_votes::dsl;
        diesel::delete(
            dsl::comment_votes
                .filter(dsl::comment_id.eq(cid))
                .filter(dsl::user_id.eq(user_id)),
        )
        .execute(conn)
        .await
        .map_err(|e| {
            TinyBoardsError::from_error_message(e, 500, "could not remove vote on comment")
        })
    }
}
