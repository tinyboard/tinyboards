use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        aggregates::BoardAggregates,
        board::boards::Board as DbBoard,
    },
    schema::{board_aggregates, board_moderators, board_subscribers, boards},
    utils::get_conn,
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

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
        let key_ids: Vec<Uuid> = keys.iter().map(|k| k.0).collect();

        let conn = &mut get_conn(&self.pool)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let results: Vec<(DbBoard, Option<BoardAggregates>)> = boards::table
            .left_join(board_aggregates::table.on(board_aggregates::board_id.eq(boards::id)))
            .filter(boards::id.eq_any(&key_ids))
            .select((boards::all_columns, board_aggregates::all_columns.nullable()))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(HashMap::from_iter(results.into_iter().map(
            |(board, agg)| (BoardId(board.id), Board::from_db(board, agg)),
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
        let key_ids: Vec<Uuid> = keys.iter().map(|k| k.0).collect();

        let conn = &mut get_conn(&self.pool)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let mod_rows: Vec<(Uuid, i32)> = board_moderators::table
            .filter(board_moderators::board_id.eq_any(&key_ids))
            .filter(board_moderators::user_id.eq(my_user_id))
            .select((board_moderators::board_id, board_moderators::permissions))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(HashMap::from_iter(
            mod_rows
                .into_iter()
                .map(|(board_id, permissions)| (ModPermsForBoardId(board_id), permissions)),
        ))
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
        let key_ids: Vec<Uuid> = keys.iter().map(|k| k.0).collect();

        let conn = &mut get_conn(&self.pool)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let subs: Vec<(Uuid, bool)> = board_subscribers::table
            .filter(board_subscribers::board_id.eq_any(&key_ids))
            .filter(board_subscribers::user_id.eq(my_user_id))
            .select((board_subscribers::board_id, board_subscribers::is_pending))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(HashMap::from_iter(subs.into_iter().map(
            |(board_id, is_pending)| {
                let sub_type = if is_pending {
                    SubscribedType::Pending
                } else {
                    SubscribedType::Subscribed
                };
                (SubscribedTypeForBoardId(board_id), sub_type)
            },
        )))
    }
}
