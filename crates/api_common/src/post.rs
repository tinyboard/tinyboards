use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SubmitPost {
    pub title: String,
    pub type_: Option<String>,
    pub url: Option<String>,
    pub body: String,
    pub creator_id: i32,
    pub board_id: i32,
    pub nsfw: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubmitPostResponse {
    pub message: String,
    pub status_code: i32,
}