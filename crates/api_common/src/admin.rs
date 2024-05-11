use serde::{Deserialize, Serialize};
use tinyboards_db_views::structs::{PersonView, RegistrationApplicationView};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddAdmin {
    pub username: String,
    pub level: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddAdminResponse {
    pub admins: Vec<PersonView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeaveAdmin {}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListBannedPersons {
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListBannedPersonsResponse {
    pub persons: Vec<PersonView>,
    pub total_count: i64,
}
