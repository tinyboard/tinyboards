use crate::schema::notifications;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = notifications)]
pub struct Notification {
    pub id: i32,
    pub user_id: i32,
    pub comment_id: i32,
    pub creation_date: NaiveDateTime,
    pub is_read: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, Insertable, AsChangeset)]
#[diesel(table_name = notifications)]
pub struct NotificationForm {
    pub user_id: Option<i32>,
    pub comment_id: Option<i32>,
}
