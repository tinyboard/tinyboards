use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use crate::schema::board_subscriber;
use chrono::NaiveDateTime;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = board_subscriber)]
pub struct BoardSubscriber {
    pub id: i32,
    pub board_id: i32,
    pub user_id: i32,
    pub published: NaiveDateTime,
    pub pending: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Insertable, AsChangeset)]
#[diesel(table_name = board_subscriber)]
pub struct BoardSubscriberForm {
    pub board_id: i32,
    pub user_id: i32,
    pub pending: Option<bool>,
}