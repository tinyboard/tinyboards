use async_graphql::dataloader::Loader;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use tinyboards_db::{
    schema::{board_moderators, post_saved, post_votes},
    utils::get_conn,
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::newtypes::{ModPermsForPostId, SavedForPostId, VoteForPostId};
use crate::PostgresLoader;

impl Loader<VoteForPostId> for PostgresLoader {
    type Value = i32;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[VoteForPostId],
    ) -> Result<
        HashMap<VoteForPostId, <Self as Loader<VoteForPostId>>::Value>,
        <Self as Loader<VoteForPostId>>::Error,
    > {
        let my_user_id = self.my_user_id;
        let key_ids: Vec<Uuid> = keys.iter().map(|id| id.0).collect();

        let conn = &mut get_conn(&self.pool)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let votes: Vec<(Uuid, i16)> = post_votes::table
            .filter(post_votes::post_id.eq_any(&key_ids))
            .filter(post_votes::user_id.eq(my_user_id))
            .select((post_votes::post_id, post_votes::score))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(HashMap::from_iter(
            votes
                .into_iter()
                .map(|(post_id, score)| (VoteForPostId(post_id), score as i32)),
        ))
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
        let key_ids: Vec<Uuid> = keys.iter().map(|id| id.0).collect();

        let conn = &mut get_conn(&self.pool)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let saved: Vec<Uuid> = post_saved::table
            .filter(post_saved::post_id.eq_any(&key_ids))
            .filter(post_saved::user_id.eq(self.my_user_id))
            .select(post_saved::post_id)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(HashMap::from_iter(
            saved.into_iter().map(|post_id| (SavedForPostId(post_id), true)),
        ))
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
        // ModPermsForPostId wraps a post_id, but mod perms are by board.
        // We'll need to look up the board_id for each post first.
        // For simplicity, look up mod rows for all boards the user moderates
        // and match them against the post's board_id.
        let my_user_id = self.my_user_id;
        let key_ids: Vec<Uuid> = keys.iter().map(|k| k.0).collect();

        let conn = &mut get_conn(&self.pool)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Get all board_ids for these posts
        use tinyboards_db::schema::posts;
        let post_boards: Vec<(Uuid, Uuid)> = posts::table
            .filter(posts::id.eq_any(&key_ids))
            .select((posts::id, posts::board_id))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let board_ids: Vec<Uuid> = post_boards.iter().map(|(_, bid)| *bid).collect();

        // Get mod permissions for these boards
        let mod_rows: Vec<(Uuid, i32)> = board_moderators::table
            .filter(board_moderators::board_id.eq_any(&board_ids))
            .filter(board_moderators::user_id.eq(my_user_id))
            .select((board_moderators::board_id, board_moderators::permissions))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let board_perms: HashMap<Uuid, i32> = mod_rows.into_iter().collect();

        Ok(HashMap::from_iter(post_boards.into_iter().filter_map(
            |(post_id, board_id)| {
                board_perms
                    .get(&board_id)
                    .map(|perms| (ModPermsForPostId(post_id), *perms))
            },
        )))
    }
}
