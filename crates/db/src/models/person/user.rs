use crate::{
    aggregates::structs::PersonAggregates,
    models::person::{local_user::LocalUser, person::Person},
};

/// Struct for combining tables for a specific user.
#[derive(Clone)]
pub struct User {
    pub person: Person,
    pub counts: PersonAggregates,
    pub local_user: Option<LocalUser>,
}
