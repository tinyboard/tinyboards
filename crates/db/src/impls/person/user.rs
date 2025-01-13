use crate::{
    aggregates::structs::PersonAggregates,
    models::person::{local_user::LocalUser, person::Person, user::User},
};

impl From<(Person, PersonAggregates, Option<LocalUser>)> for User {
    fn from((person, counts, local_user): (Person, PersonAggregates, Option<LocalUser>)) -> Self {
        Self {
            person,
            counts,
            local_user,
        }
    }
}
