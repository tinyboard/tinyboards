use serde::{Deserialize, Serialize};
use tinyboards_db_views::structs::{UserView, CommentView, PostView, BoardView};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Search {
    pub query: Option<String>,
    pub domain: Option<String>,
    pub board_id: Option<i32>,
    pub board_name: Option<String>,
    pub creator_id: Option<i32>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub sort: Option<String>,
    pub listing_type: Option<String>,
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SearchResponse {
    #[serde(rename = "type")]
    pub kind: String,
    pub comments: Vec<CommentView>,
    pub posts: Vec<PostView>,
    pub boards: Vec<BoardView>,
    pub users: Vec<UserView>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct GetFeed {
    pub listing_type: Option<String>,
    pub sort: Option<String>,
    pub creator_id: Option<i32>,
    pub board_id: Option<i32>,
    pub user_id: Option<i32>,
    pub search: Option<String>,
    pub saved_only: Option<bool>,
    pub nsfw: Option<bool>,
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetMembers {
    pub sort: Option<String>,
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetMembersResponse {
    pub members: Vec<UserView>,
}

#[derive(Serialize)]
pub struct Message {
    pub code: i32,
    pub message: String,
}

/// Generic response
impl Message {
    pub fn new(msg: &str) -> Self {
        Self {
            code: 200,
            message: String::from(msg),
        }
    }
}
