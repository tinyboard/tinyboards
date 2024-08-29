use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use tinyboards_db::models::board::{
    board_mods::BoardModerator as DbBoardMod, boards::Board as DbBoard,
};
use tinyboards_utils::TinyBoardsError;

use crate::{
    newtypes::{BoardId, ModPermsForBoardId},
    structs::boards::Board,
    PostgresLoader,
};

impl Loader<BoardId> for PostgresLoader {
    type Value = Board;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[BoardId],
    ) -> Result<HashMap<BoardId, <Self as Loader<BoardId>>::Value>, <Self as Loader<BoardId>>::Error>
    {
        let keys = keys.iter().map(|k| k.0).collect::<Vec<i32>>();

        let list = DbBoard::get_with_counts_for_ids(&self.pool, keys)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load boards."))?;

        Ok(HashMap::from_iter(list.into_iter().map(
            |(board, counts)| (board.id.into(), Board::from((board, counts))),
        )))
    }
}

impl Loader<ModPermsForBoardId> for PostgresLoader {
    type Value = i32;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[ModPermsForBoardId],
    ) -> Result<
        HashMap<ModPermsForBoardId, <Self as Loader<ModPermsForBoardId>>::Value>,
        <Self as Loader<ModPermsForBoardId>>::Error,
    > {
        let my_person_id = self.my_person_id;

        let keys = keys.into_iter().map(|k| k.0).collect::<Vec<i32>>();

        let res = DbBoardMod::get_perms_for_ids(&self.pool, keys, my_person_id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load mod permissions.")
            })?;

        Ok(HashMap::from_iter(res.into_iter().map(
            |(board_id, permissions)| (ModPermsForBoardId(board_id), permissions),
        )))
    }
}
