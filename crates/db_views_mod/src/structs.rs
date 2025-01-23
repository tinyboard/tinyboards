use serde::{Deserialize, Serialize};
use tinyboards_db::models::{
    board::boards::BoardSafe,
    comment::comments::Comment,
    moderator::{
        admin_actions::{AdminPurgeBoard, AdminPurgeComment, AdminPurgePost, AdminPurgePerson},
        mod_actions::{
            ModAddAdmin, ModAddBoard, ModAddBoardMod, ModBan, ModBanFromBoard, ModLockPost,
            ModRemoveBoard, ModRemoveComment, ModRemovePost, ModFeaturePost,
        },
    },
    post::posts::Post,
    person::person::PersonSafe,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModAddBoardView {
    pub mod_add_board: ModAddBoard,
    pub moderator: Option<PersonSafe>,
    pub board: BoardSafe,
    pub modded_user: PersonSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModAddAdminView {
    pub mod_add_admin: ModAddAdmin,
    pub moderator: Option<PersonSafe>,
    pub modded_user: PersonSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModAddBoardModView {
    pub mod_add_board_mod: ModAddBoardMod,
    pub moderator: Option<PersonSafe>,
    pub modded_user: PersonSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModBanFromBoardView {
    pub mod_ban_from_board: ModBanFromBoard,
    pub moderator: Option<PersonSafe>,
    pub board: BoardSafe,
    pub banned_user: PersonSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModBanView {
    pub mod_ban: ModBan,
    pub moderator: Option<PersonSafe>,
    pub banned_user: PersonSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModLockPostView {
    pub mod_lock_post: ModLockPost,
    pub moderator: Option<PersonSafe>,
    pub post: Post,
    pub board: BoardSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModRemoveCommentView {
    pub mod_remove_comment: ModRemoveComment,
    pub moderator: Option<PersonSafe>,
    pub comment: Comment,
    pub commenter: PersonSafe,
    pub post: Post,
    pub board: BoardSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModRemoveBoardView {
    pub mod_remove_board: ModRemoveBoard,
    pub moderator: Option<PersonSafe>,
    pub board: BoardSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModRemovePostView {
    pub mod_remove_post: ModRemovePost,
    pub moderator: Option<PersonSafe>,
    pub post: Post,
    pub board: BoardSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModFeaturePostView {
    pub mod_feature_post: ModFeaturePost,
    pub moderator: Option<PersonSafe>,
    pub post: Post,
    pub board: BoardSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminPurgeBoardView {
    pub admin_purge_board: AdminPurgeBoard,
    pub admin: Option<PersonSafe>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminPurgeCommentView {
    pub admin_purge_comment: AdminPurgeComment,
    pub admin: Option<PersonSafe>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminPurgePersonView {
    pub admin_purge_person: AdminPurgePerson,
    pub admin: Option<PersonSafe>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminPurgePostView {
    pub admin_purge_post: AdminPurgePost,
    pub admin: Option<PersonSafe>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ModLogParams {
    pub board_id: Option<i32>,
    pub mod_person_id: Option<i32>,
    pub other_person_id: Option<i32>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub hide_modlog_names: bool,
}
