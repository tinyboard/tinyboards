use serde::{Deserialize, Serialize};
use tinyboards_db_views::structs::PrivateMessageView;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreatePrivateMessage {
    pub recipient_id: i32,
    pub subject: Option<String>,
    pub body: String,    
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreatePrivateMessageResponse {
    pub message: PrivateMessageView,
}