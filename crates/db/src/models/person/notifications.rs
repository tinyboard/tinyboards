use crate::schema::notifications;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

pub enum NotificationType {
    PostReply(i32 /* id of new comment */),
    CommentReply(i32 /* id of new comment */),
    UsernameMention(i32 /* id of new comment */),
    Message(i32 /* id of new message */),
    NewPost(i32 /* id of new post */), /* TODO: bell notifications for new posts */
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = notifications)]
pub struct Notification {
    pub id: i32,
    pub kind: String,
    pub recipient_id: i32,
    pub comment_id: Option<i32>,
    pub post_id: Option<i32>,
    pub message_id: Option<i32>,
    pub created: NaiveDateTime,
    pub is_read: bool,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = notifications)]
pub struct NotificationForm {
    pub kind: Option<String>,
    pub recipient_id: Option<i32>,
    pub comment_id: Option<Option<i32>>,
    pub post_id: Option<Option<i32>>,
    pub message_id: Option<Option<i32>>,
    pub is_read: Option<bool>,
}
