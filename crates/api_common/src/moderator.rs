use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

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
    pub target_person_id: i32,
    pub board_id: i32,
    pub banned: bool,
    pub expires: Option<NaiveDateTime>,
    pub reason: Option<String>,
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
