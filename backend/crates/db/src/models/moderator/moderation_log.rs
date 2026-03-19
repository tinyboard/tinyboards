use crate::enums::DbModerationAction;
use crate::schema::moderation_log;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = moderation_log)]
pub struct ModerationLog {
    pub id: Uuid,
    pub moderator_id: Uuid,
    pub action_type: DbModerationAction,
    pub target_type: String,
    pub target_id: Uuid,
    pub board_id: Option<Uuid>,
    pub reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = moderation_log)]
pub struct ModerationLogInsertForm {
    pub moderator_id: Uuid,
    pub action_type: DbModerationAction,
    pub target_type: String,
    pub target_id: Uuid,
    pub board_id: Option<Uuid>,
    pub reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub expires_at: Option<DateTime<Utc>>,
}
