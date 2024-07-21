use async_graphql::*;
use dataloader::Loader;
use std::collections::HashMap;
use tinyboards_db::models::person::person::Person as DbPerson;
use tinyboards_db::newtypes::UserId;
use tinyboards_utils::TinyBoardsError;

use crate::{structs::person::Person, PostgresLoader};

impl Loader<UserId> for PostgresLoader {
    type Value = Person;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[UserId],
    ) -> Result<HashMap<UserId, <Self as Loader<UserId>>::Value>, <Self as Loader<UserId>>::Error>
    {
        let list = DbPerson::get_with_counts_for_ids(&self.0, keys)
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
