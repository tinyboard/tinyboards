use crate::{PostgresLoader, newtypes::UserId, structs::user::User};
use async_graphql::dataloader::Loader;
use std::collections::HashMap;
use tinyboards_db::models::user::user::User as DbUser;
use tinyboards_db::aggregates::structs::UserAggregates;
use tinyboards_utils::TinyBoardsError;

// Load item creator
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
        let keys = keys.into_iter().map(|k| k.0).collect::<Vec<i32>>();
        let list = DbUser::get_users_for_ids(&self.pool, keys)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load users."))?;

        Ok(HashMap::from_iter(
            list.into_iter()
                .map(|u| {
                    let default_counts = UserAggregates {
                        id: 0, // Default id for aggregates
                        user_id: u.id,
                        post_count: 0,
                        post_score: 0,
                        comment_count: 0,
                        comment_score: 0,
                        rep: 0,
                    };
                    (UserId(u.id), User::from((u, default_counts)))
                }),
        ))
    }
}
