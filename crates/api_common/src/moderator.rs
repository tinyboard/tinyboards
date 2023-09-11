use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tinyboards_db_views::structs::PersonView;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModActionResponse<T> {
    pub mod_action: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToggleBan {
    pub target_person_id: i32,
    pub banned: bool,
    pub expires: Option<i64>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BanFromBoard {
    pub board_id: i32,
    pub person_id: i32,
    pub ban: bool,
    pub remove_data: Option<bool>,
    pub reason: Option<String>,
    pub expires: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BanFromBoardResponse {
    pub person_view: PersonView,
    pub banned: bool,
}

#[derive(Debug, Deserialize)]
pub struct RemoveObject {
    pub target_fullname: String,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApproveObject {
    pub target_fullname: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BanBoard {
    pub board_id: i32,
    pub reason: Option<String>,
    pub banned: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PostModQueue {
    pub limit: Option<i64>,
    pub page: Option<i64>,
    pub board_id: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommentModQueue {
    pub limit: Option<i64>,
    pub page: Option<i64>,
    pub board_id: Option<i32>,
}
