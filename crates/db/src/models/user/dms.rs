use crate::schema::dms;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = dms)]
pub struct PrivateMessage {
    pub id: i32,
    pub creator_id: i32,
    pub recipient_id: i32,
    pub body: String,
    pub is_deleted: bool,
    pub read: bool,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}
