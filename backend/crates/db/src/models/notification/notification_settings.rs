use crate::schema::notification_settings;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = notification_settings)]
pub struct NotificationSettings {
    pub id: Uuid,
    pub user_id: Uuid,
    pub is_email_enabled: bool,
    pub is_comment_replies_enabled: bool,
    pub is_post_replies_enabled: bool,
    pub is_mentions_enabled: bool,
    pub is_private_messages_enabled: bool,
    pub is_board_invites_enabled: bool,
    pub is_moderator_actions_enabled: bool,
    pub is_system_notifications_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = notification_settings)]
pub struct NotificationSettingsInsertForm {
    pub user_id: Uuid,
    pub is_email_enabled: bool,
    pub is_comment_replies_enabled: bool,
    pub is_post_replies_enabled: bool,
    pub is_mentions_enabled: bool,
    pub is_private_messages_enabled: bool,
    pub is_board_invites_enabled: bool,
    pub is_moderator_actions_enabled: bool,
    pub is_system_notifications_enabled: bool,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = notification_settings)]
pub struct NotificationSettingsUpdateForm {
    pub is_email_enabled: Option<bool>,
    pub is_comment_replies_enabled: Option<bool>,
    pub is_post_replies_enabled: Option<bool>,
    pub is_mentions_enabled: Option<bool>,
    pub is_private_messages_enabled: Option<bool>,
    pub is_board_invites_enabled: Option<bool>,
    pub is_moderator_actions_enabled: Option<bool>,
    pub is_system_notifications_enabled: Option<bool>,
    pub updated_at: Option<DateTime<Utc>>,
}
