use async_graphql::*;
use dataloader::Loader;
use std::collections::HashMap;
use tinyboards_db::models::post::post_votes::PostVote as DbPostVote;
use tinyboards_db::models::{board::boards::Board as DbBoard, person::person::Person as DbPerson};
use tinyboards_db::newtypes::UserId;
use tinyboards_utils::TinyBoardsError;

use crate::newtypes::VoteForPostId;
use crate::{
    newtypes::BoardIdForPost,
    structs::{boards::Board, person::Person},
    PostgresLoader,
};

// Load post creator
impl Loader<UserId> for PostgresLoader {
    type Value = Person;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[UserId],
    ) -> Result<HashMap<UserId, <Self as Loader<UserId>>::Value>, <Self as Loader<UserId>>::Error>
    {
        let list = DbPerson::get_with_counts_for_ids(&self.pool, keys)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load users."))?;

        /*let mut h: HashMap<UserId, Person> = HashMap::new();
        let list = list.into_iter().map(Person::from).collect::<Vec<Person>>();

        for person in list.into_iter() {
            h.insert(UserId(person.id), person);
        }

        Ok(h)*/

        Ok(HashMap::from_iter(list.into_iter().map(
            |(person, counts)| (UserId(person.id), Person::from((person, counts))),
        )))
    }
}

impl Loader<BoardIdForPost> for PostgresLoader {
    type Value = Board;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[BoardIdForPost],
    ) -> Result<
        HashMap<BoardIdForPost, <Self as Loader<BoardIdForPost>>::Value>,
        <Self as Loader<BoardIdForPost>>::Error,
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
