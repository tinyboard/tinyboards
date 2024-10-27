use async_graphql::*;
use dataloader::Loader;
use std::collections::HashMap;
use tinyboards_db::models::board::board_mods::BoardModerator as DbBoardMod;
use tinyboards_db::models::post::post_saved::PostSaved as DbPostSaved;
use tinyboards_db::models::post::post_votes::PostVote as DbPostVote;
use tinyboards_utils::TinyBoardsError;

use crate::newtypes::{ModPermsForPostId, SavedForPostId, VoteForPostId};
use crate::PostgresLoader;

impl Loader<VoteForPostId> for PostgresLoader {
    type Value = i16;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[VoteForPostId],
    ) -> Result<
        HashMap<VoteForPostId, <Self as Loader<VoteForPostId>>::Value>,
        <Self as Loader<VoteForPostId>>::Error,
    > {
        let my_person_id = self.my_person_id;

        let keys = keys.into_iter().map(|id| id.0).collect::<Vec<i32>>();

        let list = DbPostVote::get_my_vote_for_ids(&self.pool, keys, my_person_id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load post votes.")
            })?;

        Ok(HashMap::from_iter(list.into_iter().map(
            |(post_id, vote_type)| (VoteForPostId(post_id), vote_type),
        )))
    }
}

impl Loader<SavedForPostId> for PostgresLoader {
    type Value = bool;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[SavedForPostId],
    ) -> Result<
        HashMap<SavedForPostId, <Self as Loader<SavedForPostId>>::Value>,
        <Self as Loader<SavedForPostId>>::Error,
    > {
        let keys = keys.into_iter().map(|id| id.0).collect::<Vec<i32>>();

        let list = DbPostSaved::get_saved_for_ids(&self.pool, keys, self.my_person_id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load saved status for post.")
            })?;

        Ok(HashMap::from_iter(list.into_iter().map(
            |(post_id, is_saved)| (SavedForPostId(post_id), is_saved),
        )))
    }
}

impl Loader<ModPermsForPostId> for PostgresLoader {
    type Value = i32;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[ModPermsForPostId],
    ) -> Result<
        HashMap<ModPermsForPostId, <Self as Loader<ModPermsForPostId>>::Value>,
        <Self as Loader<ModPermsForPostId>>::Error,
    > {
        let my_person_id = self.my_person_id;

        let keys = keys.into_iter().map(|k| k.0).collect::<Vec<i32>>();

        let res = DbBoardMod::get_perms_for_ids(&self.pool, keys, my_person_id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load mod permissions.")
            })?;

        Ok(HashMap::from_iter(res.into_iter().map(
            |(post_id, permissions)| (ModPermsForPostId(post_id), permissions),
        )))
    }
}
