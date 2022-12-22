use crate::schema::board_subscriptions;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = board_subscriptions)]
pub struct BoardSubscriber {
    pub id: i32,
    pub board_id: i32,
    pub user_id: i32,
    pub creation_date: NaiveDateTime,
    pub pending: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = board_subscriptions)]
pub struct BoardSubscriberForm {
    pub board_id: i32,
    pub user_id: i32,
    pub pending: Option<bool>,
}
