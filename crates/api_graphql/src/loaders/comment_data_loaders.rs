use async_graphql::*;
use dataloader::Loader;
use std::collections::HashMap;
use tinyboards_db::models::post::post_saved::PostSaved as DbPostSaved;
use tinyboards_db::models::post::post_votes::PostVote as DbPostVote;
use tinyboards_db::models::{
    board::boards::Board as DbBoard, person::person::Person as DbPerson,
    post::posts::Post as DbPost,
};
use tinyboards_utils::TinyBoardsError;

use crate::newtypes::{PersonId, PostIdForComment, SavedForCommentId, VoteForCommentId};
use crate::{
    newtypes::BoardIdForComment,
    structs::{boards::Board, person::Person, post::Post},
    PostgresLoader,
};

impl Loader<BoardIdForComment> for PostgresLoader {
    type Value = Board;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[BoardIdForComment],
    ) -> Result<
        HashMap<BoardIdForComment, <Self as Loader<BoardIdForComment>>::Value>,
        <Self as Loader<BoardIdForComment>>::Error,
    > {
        let keys = keys.iter().map(|k| k.0).collect::<Vec<i32>>();

        let list = DbBoard::get_with_counts_for_ids(&self.pool, keys)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load boards."))?;

        Ok(HashMap::from_iter(list.into_iter().map(
            |(board, counts)| (board.id.into(), Board::from((board, counts))),
        )))
    }
}

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
        let my_person_id = self.my_person_id;

        let keys = keys.into_iter().map(|id| id.0).collect::<Vec<i32>>();

        let list = DbPostVote::get_my_vote_for_ids(&self.pool, keys, my_person_id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load post votes.")
            })?;

        Ok(HashMap::from_iter(list.into_iter().map(
            |(post_id, vote_type)| (VoteForCommentId(post_id), vote_type),
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

        let list = DbPostSaved::get_saved_for_ids(&self.pool, keys, self.my_person_id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load saved status for post.")
            })?;

        Ok(HashMap::from_iter(list.into_iter().map(
            |(post_id, is_saved)| (SavedForCommentId(post_id), is_saved),
        )))
    }
}
