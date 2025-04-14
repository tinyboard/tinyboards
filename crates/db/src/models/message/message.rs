use crate::schema::{pm_notif, private_message};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = private_message)]
pub struct Message {
    pub id: i32,
    pub creator_id: i32,
    pub recipient_user_id: Option<i32>,
    pub recipient_board_id: Option<i32>,
    pub body: String,
    pub body_html: String,
    pub published: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub is_sender_hidden: bool,
    pub title: String,
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
    pub is_sender_hidden: bool,
    pub title: Option<String>,
}

/*#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Queryable, Identifiable)]
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
    pub recipient_id: Option<i32>,
    pub pm_id: Option<i32>,
    pub read: Option<bool>,
}*/
