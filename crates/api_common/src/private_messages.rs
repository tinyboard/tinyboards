use crate::sensitive::Sensitive;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tinyboards_db::models::user::private_messages::{PrivateMessage};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreatePrivateMessage {
    pub creator_id: i32,
    pub recipient_id: i32,
    pub subject: Option<String>,
    pub body: String,    
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreatePrivateMessageResponse {
    
}