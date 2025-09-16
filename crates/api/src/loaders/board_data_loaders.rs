use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use tinyboards_db::{
    models::board::{
        board_mods::BoardModerator as DbBoardMod, board_subscriber::BoardSubscriber as DbBoardSub,
        boards::Board as DbBoard,
    },
};
use tinyboards_utils::TinyBoardsError;

use crate::{
    PostgresLoader,
    newtypes::{BoardId, ModPermsForBoardId, SubscribedTypeForBoardId},
    structs::boards::Board,
    SubscribedType,
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
        let my_user_id = self.my_user_id;

        let keys = keys.into_iter().map(|k| k.0).collect::<Vec<i32>>();

        let res = DbBoardMod::get_perms_for_ids(&self.pool, keys, my_user_id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load mod permissions.")
            })?;

        Ok(HashMap::from_iter(res.into_iter().map(
            |(board_id, permissions)| (ModPermsForBoardId(board_id), permissions),
        )))
    }
}

impl Loader<SubscribedTypeForBoardId> for PostgresLoader {
    type Value = SubscribedType;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[SubscribedTypeForBoardId],
    ) -> Result<
        HashMap<SubscribedTypeForBoardId, <Self as Loader<SubscribedTypeForBoardId>>::Value>,
        <Self as Loader<SubscribedTypeForBoardId>>::Error,
    > {
        let my_user_id = self.my_user_id;

        let keys = keys.into_iter().map(|k| k.0).collect::<Vec<i32>>();

        let res = DbBoardSub::subscribed_type_for_ids(&self.pool, keys, my_user_id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load subscriber type.")
            })?;

        Ok(HashMap::from_iter(res.into_iter().map(
            |(board_id, subtype)| (SubscribedTypeForBoardId(board_id), subtype.into()),
        )))
    }
}
