use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use crate::schema::private_message;
use chrono::NaiveDateTime;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = private_message)]
pub struct PrivateMessage {
    pub id: i32,
    pub creator_id: i32,
    pub recipient_id: i32,
    pub body: String,
    pub deleted: bool,
    pub read: bool,
    pub published: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}