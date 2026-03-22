use crate::enums::DbNotificationKind;
use crate::schema::notifications;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = notifications)]
pub struct Notification {
    pub id: Uuid,
    pub kind: DbNotificationKind,
    pub recipient_user_id: Uuid,
    pub comment_id: Option<Uuid>,
    pub post_id: Option<Uuid>,
    pub message_id: Option<Uuid>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub actor_user_id: Option<Uuid>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = notifications)]
pub struct NotificationInsertForm {
    pub kind: DbNotificationKind,
    pub recipient_user_id: Uuid,
    pub comment_id: Option<Uuid>,
    pub post_id: Option<Uuid>,
    pub message_id: Option<Uuid>,
    pub is_read: bool,
    pub actor_user_id: Option<Uuid>,
}
