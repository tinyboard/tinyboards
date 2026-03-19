use crate::schema::private_messages;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = private_messages)]
pub struct PrivateMessage {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub recipient_id: Option<Uuid>,
    pub recipient_board_id: Option<Uuid>,
    pub subject: Option<String>,
    pub body: String,
    pub body_html: String,
    pub is_read: bool,
    pub is_sender_hidden: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = private_messages)]
pub struct PrivateMessageInsertForm {
    pub creator_id: Uuid,
    pub recipient_id: Option<Uuid>,
    pub recipient_board_id: Option<Uuid>,
    pub subject: Option<String>,
    pub body: String,
    pub body_html: String,
    pub is_read: bool,
    pub is_sender_hidden: bool,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = private_messages)]
pub struct PrivateMessageUpdateForm {
    pub subject: Option<Option<String>>,
    pub body: Option<String>,
    pub body_html: Option<String>,
    pub is_read: Option<bool>,
    pub is_sender_hidden: Option<bool>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<Option<DateTime<Utc>>>,
}
