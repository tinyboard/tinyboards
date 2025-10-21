use crate::schema::notification_settings;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = notification_settings)]
pub struct NotificationSettings {
    pub id: i32,
    pub user_id: i32,
    pub email_enabled: bool,
    pub comment_replies_enabled: bool,
    pub post_replies_enabled: bool,
    pub mentions_enabled: bool,
    pub private_messages_enabled: bool,
    pub board_invites_enabled: bool,
    pub moderator_actions_enabled: bool,
    pub system_notifications_enabled: bool,
    pub created: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
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
#[diesel(table_name = notification_settings)]
pub struct NotificationSettingsForm {
    pub user_id: i32,
    pub email_enabled: Option<bool>,
    pub comment_replies_enabled: Option<bool>,
    pub post_replies_enabled: Option<bool>,
    pub mentions_enabled: Option<bool>,
    pub private_messages_enabled: Option<bool>,
    pub board_invites_enabled: Option<bool>,
    pub moderator_actions_enabled: Option<bool>,
    pub system_notifications_enabled: Option<bool>,
    pub created: Option<NaiveDateTime>,
    pub updated: Option<Option<NaiveDateTime>>,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            id: 0,
            user_id: 0,
            email_enabled: true,
            comment_replies_enabled: true,
            post_replies_enabled: true,
            mentions_enabled: true,
            private_messages_enabled: true,
            board_invites_enabled: true,
            moderator_actions_enabled: true,
            system_notifications_enabled: true,
            created: chrono::Utc::now().naive_utc(),
            updated: None,
        }
    }
}