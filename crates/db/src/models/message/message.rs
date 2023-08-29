use crate::schema::{pm_notif, private_message};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = private_message)]
pub struct Message {
    id: i32,
    creator_id: i32,
    recipient_user_id: Option<i32>,
    recipient_board_id: Option<i32>,
    body: String,
    body_html: String,
    published: NaiveDateTime,
    updated: Option<NaiveDateTime>,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = private_message)]
pub struct MessageForm {
    pub creator_id: Option<i32>,
    pub recipient_user_id: Option<Option<i32>>,
    pub recipient_board_id: Option<Option<i32>>,
    pub body: Option<String>,
    pub body_html: Option<String>,
    pub updated: Option<Option<NaiveDateTime>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = pm_notif)]
pub struct MessageNotif {
    id: i32,
    recipient_id: i32,
    pm_id: i32,
    read: bool,
    creation_date: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = pm_notif)]
pub struct MessageNotifForm {
    recipient_id: Option<i32>,
    pm_id: Option<i32>,
    read: Option<bool>,
}
