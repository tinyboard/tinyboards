use crate::schema::notifications;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = notifications)]
pub struct Notification {
    pub id: i32,
    pub kind: String,
    pub recipient_user_id: i32,
    pub comment_id: Option<i32>,
    pub post_id: Option<i32>,
    pub message_id: Option<i32>,
    pub created: NaiveDateTime,
    pub is_read: bool,
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Debug,
    Serialize,
    Deserialize,
    Default,
    Insertable,
    AsChangeset,
)]
#[diesel(table_name = notifications)]
pub struct NotificationForm {
    pub kind: String,
    pub recipient_user_id: i32,
    pub comment_id: Option<i32>,
    pub post_id: Option<i32>,
    pub message_id: Option<i32>,
    pub created: Option<NaiveDateTime>,
    pub is_read: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationKind {
    CommentReply,
    PostReply,
    Mention,
    PostVote,
    CommentVote,
    PrivateMessage,
    BoardInvite,
    ModeratorAction,
    SystemNotification,
}

impl std::fmt::Display for NotificationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationKind::CommentReply => write!(f, "comment_reply"),
            NotificationKind::PostReply => write!(f, "post_reply"),
            NotificationKind::Mention => write!(f, "mention"),
            NotificationKind::PostVote => write!(f, "post_vote"),
            NotificationKind::CommentVote => write!(f, "comment_vote"),
            NotificationKind::PrivateMessage => write!(f, "private_message"),
            NotificationKind::BoardInvite => write!(f, "board_invite"),
            NotificationKind::ModeratorAction => write!(f, "moderator_action"),
            NotificationKind::SystemNotification => write!(f, "system_notification"),
        }
    }
}

impl From<&str> for NotificationKind {
    fn from(s: &str) -> Self {
        match s {
            "comment_reply" => NotificationKind::CommentReply,
            "post_reply" => NotificationKind::PostReply,
            "mention" => NotificationKind::Mention,
            "post_vote" => NotificationKind::PostVote,
            "comment_vote" => NotificationKind::CommentVote,
            "private_message" => NotificationKind::PrivateMessage,
            "board_invite" => NotificationKind::BoardInvite,
            "moderator_action" => NotificationKind::ModeratorAction,
            "system_notification" => NotificationKind::SystemNotification,
            _ => NotificationKind::SystemNotification, // fallback
        }
    }
}

impl From<String> for NotificationKind {
    fn from(s: String) -> Self {
        NotificationKind::from(s.as_str())
    }
}