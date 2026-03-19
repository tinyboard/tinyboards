use async_graphql::dataloader::Loader;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use tinyboards_db::{
    models::{
        aggregates::PostAggregates,
        post::posts::Post as DbPost,
    },
    schema::{comment_saved, comment_votes, post_aggregates, posts},
    utils::get_conn,
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::newtypes::{PostIdForComment, SavedForCommentId, VoteForCommentId};
use crate::{PostgresLoader, structs::post::Post};

impl Loader<PostIdForComment> for PostgresLoader {
    type Value = Post;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[PostIdForComment],
    ) -> Result<
        HashMap<PostIdForComment, <Self as Loader<PostIdForComment>>::Value>,
        <Self as Loader<PostIdForComment>>::Error,
    > {
        let key_ids: Vec<Uuid> = keys.iter().map(|k| k.0).collect();

        let conn = &mut get_conn(&self.pool)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let results: Vec<(DbPost, PostAggregates)> = posts::table
            .inner_join(post_aggregates::table.on(post_aggregates::post_id.eq(posts::id)))
            .filter(posts::id.eq_any(&key_ids))
            .select((posts::all_columns, post_aggregates::all_columns))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(HashMap::from_iter(results.into_iter().map(
            |(post, counts)| (PostIdForComment(post.id), Post::from((post, counts))),
        )))
    }
}

impl Loader<VoteForCommentId> for PostgresLoader {
    type Value = i32;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[VoteForCommentId],
    ) -> Result<
        HashMap<VoteForCommentId, <Self as Loader<VoteForCommentId>>::Value>,
        <Self as Loader<VoteForCommentId>>::Error,
    > {
        let my_user_id = self.my_user_id;
        let key_ids: Vec<Uuid> = keys.iter().map(|id| id.0).collect();

        let conn = &mut get_conn(&self.pool)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let votes: Vec<(Uuid, i16)> = comment_votes::table
            .filter(comment_votes::comment_id.eq_any(&key_ids))
            .filter(comment_votes::user_id.eq(my_user_id))
            .select((comment_votes::comment_id, comment_votes::score))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(HashMap::from_iter(
            votes
                .into_iter()
                .map(|(comment_id, score)| (VoteForCommentId(comment_id), score as i32)),
        ))
    }
}

impl Loader<SavedForCommentId> for PostgresLoader {
    type Value = bool;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[SavedForCommentId],
    ) -> Result<
        HashMap<SavedForCommentId, <Self as Loader<SavedForCommentId>>::Value>,
        <Self as Loader<SavedForCommentId>>::Error,
    > {
        let key_ids: Vec<Uuid> = keys.iter().map(|id| id.0).collect();

        let conn = &mut get_conn(&self.pool)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let saved: Vec<Uuid> = comment_saved::table
            .filter(comment_saved::comment_id.eq_any(&key_ids))
            .filter(comment_saved::user_id.eq(self.my_user_id))
            .select(comment_saved::comment_id)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(HashMap::from_iter(
            saved.into_iter().map(|comment_id| (SavedForCommentId(comment_id), true)),
        ))
    }
}
