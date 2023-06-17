use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tinyboards_db_views::structs::PersonView;

use crate::sensitive::Sensitive;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModActionResponse<T> {
    pub mod_action: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StickyPost {
    pub post_id: i32,
    pub stickied: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BanUser {
    pub target_person_id: i32,
    pub banned: bool,
    pub expires: Option<NaiveDateTime>,
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

#[derive(Debug, Deserialize)]
pub struct LockObject {
    pub target_fullname: String,
}

#[derive(Debug, Deserialize)]
pub struct UnlockObject {
    pub target_fullname: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BanBoard {
    pub board_id: i32,
    pub reason: Option<String>,
    pub banned: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddBoardMod {
    pub added: bool,
    pub added_person_id: i32,
    pub added_board_id: i32,
}
