use crate::schema::comment_reply;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = comment_reply)]
pub struct CommentReply {
    pub id: i32,
    pub recipient_id: i32,
    pub comment_id: i32,
    pub read: bool,
    pub creation_date: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = comment_reply)]
pub struct CommentReplyForm {
    pub recipient_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub read: Option<bool>,
}