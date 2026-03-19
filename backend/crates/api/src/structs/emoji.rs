use async_graphql::*;
use tinyboards_db::models::emoji::Emoji as DbEmoji;

#[derive(SimpleObject, Clone)]
pub struct EmojiObject {
    pub id: ID,
    pub shortcode: String,
    pub image_url: String,
    pub alt_text: String,
    pub category: String,
    pub scope: String,
    pub board_id: Option<ID>,
    pub created_by: ID,
    pub is_active: bool,
    pub usage_count: i32,
    #[graphql(name = "createdAt")]
    pub created_at: String,
    #[graphql(name = "updatedAt")]
    pub updated_at: String,
}

impl From<DbEmoji> for EmojiObject {
    fn from(emoji: DbEmoji) -> Self {
        let scope_str = match emoji.scope {
            tinyboards_db::enums::DbEmojiScope::Global => "site",
            tinyboards_db::enums::DbEmojiScope::Board => "board",
        };
        Self {
            id: emoji.id.to_string().into(),
            shortcode: emoji.shortcode,
            image_url: emoji.image_url,
            alt_text: emoji.alt_text,
            category: emoji.category,
            scope: scope_str.to_string(),
            board_id: emoji.board_id.map(|id| id.to_string().into()),
            created_by: emoji.created_by.to_string().into(),
            is_active: emoji.is_active,
            usage_count: emoji.usage_count,
            created_at: emoji.created_at.to_string(),
            updated_at: emoji.updated_at.to_string(),
        }
    }
}

#[derive(InputObject)]
pub struct CreateEmojiInput {
    pub shortcode: String,
    pub image_url: String,
    pub alt_text: String,
    pub category: String,
    pub board_id: Option<ID>,
    pub scope: Option<EmojiScope>,
    pub keywords: Option<Vec<String>>,
}

#[derive(InputObject)]
pub struct UpdateEmojiInput {
    pub shortcode: Option<String>,
    pub image_url: Option<String>,
    pub alt_text: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
    pub keywords: Option<Vec<String>>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum EmojiScope {
    Site,
    Board,
}

#[derive(InputObject)]
pub struct ListEmojisInput {
    pub board_id: Option<ID>,
    pub scope: Option<EmojiScope>,
    pub category: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

impl Default for ListEmojisInput {
    fn default() -> Self {
        Self {
            board_id: None,
            scope: None,
            category: None,
            search: None,
            limit: Some(50),
            offset: Some(0),
        }
    }
}
