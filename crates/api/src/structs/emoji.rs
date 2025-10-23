use async_graphql::*;
use tinyboards_db::models::emoji::emoji::Emoji;

#[derive(SimpleObject, Clone)]
pub struct EmojiObject {
    pub id: i32,
    pub shortcode: String,
    pub image_url: String,
    pub alt_text_display: String,
    pub category: String,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub board_id: Option<i32>,
    pub created_by_user_id: i32,
    pub is_active: bool,
    pub usage_count: i32,
    pub emoji_scope: String,
}

impl From<Emoji> for EmojiObject {
    fn from(emoji: Emoji) -> Self {
        Self {
            id: emoji.id,
            shortcode: emoji.shortcode,
            image_url: emoji.image_url.to_string(),
            alt_text_display: emoji.alt_text,
            category: emoji.category,
            created_at: emoji.creation_date.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
            updated_at: emoji.updated.map(|u| u.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string()),
            board_id: emoji.board_id,
            created_by_user_id: emoji.created_by_user_id,
            is_active: emoji.is_active,
            usage_count: emoji.usage_count,
            emoji_scope: emoji.emoji_scope,
        }
    }
}

#[derive(InputObject)]
pub struct CreateEmojiInput {
    pub shortcode: String,
    pub image_file: Upload,
    pub alt_text_display: String,
    pub category: String,
    pub board_id: Option<i32>,
    pub emoji_scope: Option<EmojiScope>,
}

#[derive(InputObject)]
pub struct UpdateEmojiInput {
    pub shortcode: Option<String>,
    pub image_file: Option<Upload>,
    pub alt_text_display: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum EmojiScope {
    Site,
    Board,
}

#[derive(InputObject)]
pub struct ListEmojisInput {
    pub board_id: Option<i32>,
    pub scope: Option<EmojiScope>,
    pub active_only: Option<bool>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub search: Option<String>,
}