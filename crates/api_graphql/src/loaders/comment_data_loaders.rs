use async_graphql::*;
use dataloader::Loader;
use std::collections::HashMap;
use tinyboards_db::models::comment::{
    comment_saved::CommentSaved as DbCommentSaved, comment_votes::CommentVote as DbCommentVote,
};
use tinyboards_db::models::post::posts::Post as DbPost;
use tinyboards_utils::TinyBoardsError;

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
        let keys = keys.iter().map(|k| k.0).collect::<Vec<i32>>();

        let list = DbPost::load_with_counts_for_ids(&self.pool, keys, false)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load posts."))?;

        Ok(HashMap::from_iter(list.into_iter().map(
            |(post, counts)| (post.id.into(), Post::from((post, counts))),
        )))
    }
}

impl Loader<VoteForCommentId> for PostgresLoader {
    type Value = i16;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[VoteForCommentId],
    ) -> Result<
        HashMap<VoteForCommentId, <Self as Loader<VoteForCommentId>>::Value>,
        <Self as Loader<VoteForCommentId>>::Error,
    > {
        let my_user_id = self.my_user_id;

        let keys = keys.into_iter().map(|id| id.0).collect::<Vec<i32>>();

        let list = DbCommentVote::get_my_vote_for_ids(&self.pool, keys, my_user_id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load comment votes.")
            })?;

        Ok(HashMap::from_iter(list.into_iter().map(
            |(comment_id, vote_type)| (VoteForCommentId(comment_id), vote_type),
        )))
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
        let keys = keys.into_iter().map(|id| id.0).collect::<Vec<i32>>();

        let list = DbCommentSaved::get_saved_for_ids(&self.pool, keys, self.my_user_id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(
                    e,
                    500,
                    "Failed to load saved status for comment.",
                )
            })?;

        Ok(HashMap::from_iter(list.into_iter().map(
            |(comment_id, is_saved)| (SavedForCommentId(comment_id), is_saved),
        )))
    }
}
