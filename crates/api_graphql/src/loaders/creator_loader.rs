use crate::{newtypes::PersonId, structs::person::Person, PostgresLoader};
use async_graphql::dataloader::Loader;
use std::collections::HashMap;
use tinyboards_db::models::person::person::Person as DbPerson;
use tinyboards_utils::TinyBoardsError;

// Load item creator
impl Loader<PersonId> for PostgresLoader {
    type Value = Person;
    type Error = TinyBoardsError;

    async fn load(
        &self,
        keys: &[PersonId],
    ) -> Result<
        HashMap<PersonId, <Self as Loader<PersonId>>::Value>,
        <Self as Loader<PersonId>>::Error,
    > {
        let keys = keys.into_iter().map(|k| k.0).collect::<Vec<i32>>();
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
            |(person, counts)| (PersonId(person.id), Person::from((person, counts))),
        )))
    }
}
