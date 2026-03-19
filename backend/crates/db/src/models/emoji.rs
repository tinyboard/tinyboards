use crate::enums::DbEmojiScope;
use crate::schema::{emoji, emoji_keywords};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// emoji
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = emoji)]
pub struct Emoji {
    pub id: Uuid,
    pub shortcode: String,
    pub image_url: String,
    pub alt_text: String,
    pub category: String,
    pub scope: DbEmojiScope,
    pub board_id: Option<Uuid>,
    pub created_by: Uuid,
    pub is_active: bool,
    pub usage_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = emoji)]
pub struct EmojiInsertForm {
    pub shortcode: String,
    pub image_url: String,
    pub alt_text: String,
    pub category: String,
    pub scope: DbEmojiScope,
    pub board_id: Option<Uuid>,
    pub created_by: Uuid,
    pub is_active: bool,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = emoji)]
pub struct EmojiUpdateForm {
    pub shortcode: Option<String>,
    pub image_url: Option<String>,
    pub alt_text: Option<String>,
    pub category: Option<String>,
    pub scope: Option<DbEmojiScope>,
    pub board_id: Option<Option<Uuid>>,
    pub is_active: Option<bool>,
}

// ============================================================
// emoji_keywords
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = emoji_keywords)]
pub struct EmojiKeyword {
    pub id: Uuid,
    pub emoji_id: Uuid,
    pub keyword: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = emoji_keywords)]
pub struct EmojiKeywordInsertForm {
    pub emoji_id: Uuid,
    pub keyword: String,
}
