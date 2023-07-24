use serde::{Deserialize, Serialize};
use tinyboards_db::newtypes::DbUrl;
use tinyboards_db_views::structs::EmojiView;

#[derive(Debug, Serialize, Deserialize, Clone)]
/// create a custom emoji for the local instance.
pub struct CreateEmoji {
    pub category: String,
    pub shortcode: String,
    pub image_url: DbUrl,
    pub alt_text: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// edit an existing custom emoji.
pub struct EditEmoji {
    pub id: i32,
    pub category: String,
    pub image_url: DbUrl,
    pub alt_text: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
/// delete a custom emoji.
pub struct DeleteEmoji {
    pub id: i32,
}


#[derive(Serialize, Deserialize, Clone)]
/// the response for deleting an emoji.
pub struct DeleteEmojiResponse {
    pub id: i32,
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// the response after creating or editing a custom emoji.
pub struct EmojiResponse {
    pub emoji: EmojiView,
}