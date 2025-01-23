use crate::schema::person_subscriber;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = person_subscriber)]
pub struct PersonSubscriber {
    pub id: i32,
    pub person_id: i32,
    pub subscriber_id: i32,
    pub creation_id: NaiveDateTime,
    pub pending: bool,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = person_subscriber)]
pub struct PersonSubscriberForm {
    pub person_id: i32,
    pub subscriber_id: i32,
    pub pending: bool
}