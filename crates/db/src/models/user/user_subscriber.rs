use crate::schema::user_subscriber;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_subscriber)]
pub struct UserSubscriber {
    pub id: i32,
    pub user_id: i32,
    pub subscriber_id: i32,
    pub creation_date: NaiveDateTime,
    pub pending: bool,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = user_subscriber)]
pub struct UserSubscriberForm {
    pub user_id: Option<i32>,
    pub subscriber_id: Option<i32>,
    pub pending: Option<bool>,
    pub creation_date: Option<NaiveDateTime>,
}