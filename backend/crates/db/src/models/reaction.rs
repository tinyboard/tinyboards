use crate::schema::{reactions, board_reaction_settings};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// reactions
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = reactions)]
pub struct Reaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub post_id: Option<Uuid>,
    pub comment_id: Option<Uuid>,
    pub emoji: String,
    pub score: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = reactions)]
pub struct ReactionInsertForm {
    pub user_id: Uuid,
    pub post_id: Option<Uuid>,
    pub comment_id: Option<Uuid>,
    pub emoji: String,
    pub score: i32,
}

// ============================================================
// board_reaction_settings
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = board_reaction_settings)]
pub struct BoardReactionSettings {
    pub id: Uuid,
    pub board_id: Uuid,
    pub emoji_weights: serde_json::Value,
    pub is_reactions_enabled: bool,
    pub reaction_emojis: serde_json::Value,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = board_reaction_settings)]
pub struct BoardReactionSettingsInsertForm {
    pub board_id: Uuid,
    pub emoji_weights: serde_json::Value,
    pub is_reactions_enabled: bool,
    pub reaction_emojis: serde_json::Value,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = board_reaction_settings)]
pub struct BoardReactionSettingsUpdateForm {
    pub emoji_weights: Option<serde_json::Value>,
    pub is_reactions_enabled: Option<bool>,
    pub reaction_emojis: Option<serde_json::Value>,
}
