use crate::schema::private_messages;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = private_messages)]
pub struct PrivateMessage {
    pub id: i32,
    pub creator_id: i32,
    pub recipient_id: i32,
    pub subject: Option<String>,
    pub body: String,
    pub is_deleted: bool,
    pub read: bool,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, Insertable, AsChangeset)]
#[diesel(table_name = private_messages)]
pub struct PrivateMessageForm {
    pub creator_id: Option<i32>,
    pub recipient_id: Option<i32>,
    pub subject: Option<Option<String>>,
    pub body: Option<String>,
    pub is_deleted: Option<bool>,
    pub read: Option<bool>,
    pub updated: Option<NaiveDateTime>,
}