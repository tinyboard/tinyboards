use tinyboards_db::{
    models::{
        comment::comment::Comment,
        board::board::BoardSafe,
        moderator::{
            mod_actions::{
                ModAdd,
                ModAddBoard,
                ModBan,
                ModBanFromBoard,
                ModLockPost,
                ModRemoveComment,
                ModRemoveBoard,
                ModRemovePost,
                ModStickyPost,
            },
            admin_actions::{
                AdminPurgeComment,
                AdminPurgeBoard,
                AdminPurgeUser,
                AdminPurgePost,
            },
        },
        user::user::UserSafe,
        post::post::Post,
    },
};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModAddBoardView {
    pub mod_add_board: ModAddBoard,
    pub moderator: Option<UserSafe>,
    pub board: BoardSafe,
    pub modded_user: UserSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModAddView {
    pub mod_add: ModAdd,
    pub moderator: Option<UserSafe>,
    pub modded_user: UserSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModBanFromBoardView {
    pub mod_ban_from_board: ModBanFromBoard,
    pub moderator: Option<UserSafe>,
    pub board: BoardSafe,
    pub banned_user: UserSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModBanView {
    pub mod_ban: ModBan,
    pub moderator: Option<UserSafe>,
    pub banned_user: UserSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModLockPostView {
    pub mod_lock_post: ModLockPost,
    pub moderator: Option<UserSafe>,
    pub post: Post,
    pub board: BoardSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModRemoveCommentView {
    pub mod_remove_comment: ModRemoveComment,
    pub moderator: Option<UserSafe>,
    pub comment: Comment,
    pub commenter: UserSafe,
    pub post: Post,
    pub board: BoardSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModRemoveBoardView {
    pub mod_remove_board: ModRemoveBoard,
    pub moderator: Option<UserSafe>,
    pub board: BoardSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModRemovePostView {
    pub mod_remove_post: ModRemovePost,
    pub moderator: Option<UserSafe>,
    pub post: Post,
    pub board: BoardSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModStickyPostView {
    pub mod_sticky_post: ModStickyPost,
    pub moderator: Option<UserSafe>,
    pub post: Post,
    pub board: BoardSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminPurgeBoardView {
    pub admin_purge_board: AdminPurgeBoard,
    pub admin: Option<UserSafe>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminPurgeCommentView {
    pub admin_purge_comment: AdminPurgeComment,
    pub admin: Option<UserSafe>,
    pub post: Post,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminPurgeUserView {
    pub admin_purge_user: AdminPurgeUser,
    pub admin: Option<UserSafe>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminPurgePostView {
    pub admin_purge_post: AdminPurgePost,
    pub admin: Option<UserSafe>,
    pub board: BoardSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ModLogParams {
    pub board_id: Option<i32>,
    pub mod_user_id: Option<i32>,
    pub other_user_id: Option<i32>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub hide_modlog_names: bool,
}