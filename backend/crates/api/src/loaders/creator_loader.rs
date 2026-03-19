use crate::{PostgresLoader, newtypes::UserId, structs::user::User};
use async_graphql::dataloader::Loader;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use tinyboards_db::{
    models::{
        aggregates::UserAggregates,
        user::user::User as DbUser,
    },
    schema::{user_aggregates, users},
    utils::get_conn,
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

impl Loader<UserId> for PostgresLoader {
    type Value = User;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[UserId],
    ) -> Result<
        HashMap<UserId, <Self as Loader<UserId>>::Value>,
        <Self as Loader<UserId>>::Error,
    > {
        let key_ids: Vec<Uuid> = keys.iter().map(|k| k.0).collect();

        let conn = &mut get_conn(&self.pool)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let results: Vec<(DbUser, Option<UserAggregates>)> = users::table
            .left_join(user_aggregates::table.on(user_aggregates::user_id.eq(users::id)))
            .filter(users::id.eq_any(&key_ids))
            .select((users::all_columns, user_aggregates::all_columns.nullable()))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(HashMap::from_iter(
            results.into_iter().map(|(u, aggregates)| {
                let uid = u.id;
                let user = User::from_db(u, aggregates);
                (UserId(uid), user)
            }),
        ))
    }
}
