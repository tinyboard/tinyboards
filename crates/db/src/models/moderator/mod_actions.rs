use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::schema::{
    mod_add_board,
    mod_add_admin,
    mod_add_board_mod,
    mod_ban_from_board,
    mod_ban,
    mod_lock_post,
    mod_remove_board,
    mod_remove_comment,
    mod_remove_post,
    mod_sticky_post,
};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = mod_add_board)]
pub struct ModAddBoard {
    pub id: i32,
    pub mod_user_id: i32,
    pub other_user_id: i32,
    pub board_id: i32,
    pub removed: Option<bool>,
    pub when_: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = mod_add_board)]
pub struct ModAddBoardForm {
    pub mod_user_id: i32,
    pub other_user_id: i32,
    pub board_id: i32,
    pub removed: Option<Option<bool>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = mod_add_admin)]
pub struct ModAddAdmin {
    pub id: i32,
    pub mod_user_id: i32,
    pub other_user_id: i32,
    pub removed: Option<bool>,
    pub when_: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = mod_add_admin)]
pub struct ModAddAdminForm {
    pub mod_user_id: i32,
    pub other_user_id: i32,
    pub removed: Option<Option<bool>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = mod_add_board_mod)]
pub struct ModAddBoardMod {
    pub id: i32,
    pub mod_user_id: i32,
    pub other_user_id: i32,
    pub board_id: i32,
    pub removed: Option<bool>,
    pub when_: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = mod_add_board_mod)]
pub struct ModAddBoardModForm {
    pub mod_user_id: i32,
    pub other_user_id: i32,
    pub board_id: i32,
    pub removed: Option<Option<bool>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = mod_ban_from_board)]
pub struct ModBanFromBoard {
    pub id: i32,
    pub mod_user_id: i32,
    pub other_user_id: i32,
    pub board_id: i32,
    pub reason: Option<String>,
    pub banned: Option<bool>,
    pub expires: Option<NaiveDateTime>,
    pub when_: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = mod_ban_from_board)]
pub struct ModBanFromBoardForm {
    pub mod_user_id: i32,
    pub other_user_id: i32,
    pub board_id: i32,
    pub reason: Option<Option<String>>,
    pub banned: Option<Option<bool>>,
    pub expires: Option<Option<NaiveDateTime>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = mod_ban)]
pub struct ModBan {
    pub id: i32,
    pub mod_user_id: i32,
    pub other_user_id: i32,
    pub reason: Option<String>,
    pub banned: Option<bool>,
    pub expires: Option<NaiveDateTime>,
    pub when_: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = mod_ban)]
pub struct ModBanForm {
    pub mod_user_id: i32,
    pub other_user_id: i32,
    pub reason: Option<Option<String>>,
    pub banned: Option<Option<bool>>,
    pub expires: Option<Option<NaiveDateTime>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = mod_lock_post)]
pub struct ModLockPost {
    pub id: i32,
    pub mod_user_id: i32,
    pub post_id: i32,
    pub locked: Option<bool>,
    pub when_: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = mod_lock_post)]
pub struct ModLockPostForm {
    pub mod_user_id: i32,
    pub post_id: i32,
    pub locked: Option<Option<bool>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = mod_remove_board)]
pub struct ModRemoveBoard {
    pub id: i32,
    pub mod_user_id: i32,
    pub board_id: i32,
    pub reason: Option<String>,
    pub removed: Option<bool>,
    pub when_: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = mod_remove_board)]
pub struct ModRemoveBoardForm {
    pub mod_user_id: i32,
    pub board_id: i32,
    pub reason: Option<Option<String>>,
    pub removed: Option<Option<bool>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = mod_remove_comment)]
pub struct ModRemoveComment {
    pub id: i32,
    pub mod_user_id: i32,
    pub comment_id: i32,
    pub reason: Option<String>,
    pub removed: Option<bool>,
    pub when_: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = mod_remove_comment)]
pub struct ModRemoveCommentForm {
    pub mod_user_id: i32,
    pub comment_id: i32,
    pub reason: Option<Option<String>>,
    pub removed: Option<Option<bool>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = mod_remove_post)]
pub struct ModRemovePost {
    pub id: i32,
    pub mod_user_id: i32,
    pub post_id: i32,
    pub reason: Option<String>,
    pub removed: Option<bool>,
    pub when_: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = mod_remove_post)]
pub struct ModRemovePostForm {
    pub mod_user_id: i32,
    pub post_id: i32,
    pub reason: Option<Option<String>>,
    pub removed: Option<Option<bool>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = mod_sticky_post)]
pub struct ModStickyPost {
    pub id: i32,
    pub mod_user_id: i32,
    pub post_id: i32,
    pub stickied: Option<bool>,
    pub when_: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = mod_sticky_post)]
pub struct ModStickyPostForm {
    pub mod_user_id: i32,
    pub post_id: i32,
    pub stickied: Option<Option<bool>>,
}