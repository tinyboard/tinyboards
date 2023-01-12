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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetPrivateMessages {
    pub unread_only: Option<bool>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivateMessagesResponse {
  pub messages: Vec<PrivateMessageView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivateMessageResponse {
  pub message: PrivateMessageView,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EditPrivateMessage {
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeletePrivateMessage {
    pub is_deleted: bool,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivateMessageIdPath {
    pub pm_id: i32,
}