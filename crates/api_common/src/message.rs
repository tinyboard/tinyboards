use serde::{Deserialize, Serialize};
use tinyboards_db_views::structs::MessageView;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetMessages {
    pub board_id: Option<i32>,
    pub unread_only: Option<bool>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetMessagesResponse {
    pub messages: Vec<MessageView>,
    pub count: i64,
    pub unread: i64,
}
