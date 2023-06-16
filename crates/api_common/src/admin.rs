use serde::{Serialize, Deserialize};
use tinyboards_db_views::structs::RegistrationApplicationView;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PurgePost {
    pub post_id: i32,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PurgeComment {
    pub comment_id: i32,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PurgeBoard {
    pub board_id: i32,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PurgePerson {
    pub person_id: i32,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PurgeItemResponse {
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApplicationIdPath {
    pub app_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HandleRegistrationApplication {
    pub approve: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HandleRegistrationApplicationResponse {
    pub application: Option<RegistrationApplicationView>,
}