// use tinyboards_db::{
//     models::moderator::mod_actions,
// };

// use tinyboards_db_views_mod::{
//     structs,
//     mod_add_board_view,
//     mod_add_view,
//     mod_ban_from_board_view,
//     mod_lock_post_view,
//     mod_remove_board_view,
//     mod_remove_comment_view,
//     mod_remove_post_view,
//     mod_sticky_post_view,
// };
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModActionResponse<T> {
    pub mod_action: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockPost {
    pub post_id: i32,
    pub locked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BanUser {
    pub mod_user_id: i32,
    pub other_user_id: i32,
    pub banned: bool,
    pub expires: Option<Option<NaiveDateTime>>,
    pub reason: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemovePost {
    pub mod_user_id: i32,
    pub post_id: i32,
    pub reason: Option<String>,
    pub removed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoveComment {
    pub mod_user_id: i32,
    pub comment_id: i32,
    pub reason: Option<String>,
    pub removed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoveBoard {
    pub mod_user_id: i32,
    pub board_id: i32,
    pub reason: Option<String>,
    pub removed: bool,
}