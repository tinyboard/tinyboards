use crate::schema::{
    board_blocks, board_languages, board_subscribers, board_user_bans, comment_saved, post_hidden,
    post_saved, site_languages, user_bans, user_blocks, user_follows, user_languages,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// board_subscribers
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = board_subscribers)]
pub struct BoardSubscriber {
    pub id: Uuid,
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub is_pending: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = board_subscribers)]
pub struct BoardSubscriberInsertForm {
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub is_pending: bool,
}

// ============================================================
// board_user_bans
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = board_user_bans)]
pub struct BoardUserBan {
    pub id: Uuid,
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = board_user_bans)]
pub struct BoardUserBanInsertForm {
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
}

// ============================================================
// user_bans
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_bans)]
pub struct UserBan {
    pub id: Uuid,
    pub user_id: Uuid,
    pub banned_by: Option<Uuid>,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub banned_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = user_bans)]
pub struct UserBanInsertForm {
    pub user_id: Uuid,
    pub banned_by: Option<Uuid>,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub banned_at: DateTime<Utc>,
}

// ============================================================
// user_blocks
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_blocks)]
pub struct UserBlock {
    pub id: Uuid,
    pub user_id: Uuid,
    pub target_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = user_blocks)]
pub struct UserBlockInsertForm {
    pub user_id: Uuid,
    pub target_id: Uuid,
}

// ============================================================
// board_blocks
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = board_blocks)]
pub struct BoardBlock {
    pub id: Uuid,
    pub user_id: Uuid,
    pub board_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = board_blocks)]
pub struct BoardBlockInsertForm {
    pub user_id: Uuid,
    pub board_id: Uuid,
}

// ============================================================
// post_saved
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = post_saved)]
pub struct PostSaved {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = post_saved)]
pub struct PostSavedInsertForm {
    pub post_id: Uuid,
    pub user_id: Uuid,
}

// ============================================================
// comment_saved
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = comment_saved)]
pub struct CommentSaved {
    pub id: Uuid,
    pub comment_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = comment_saved)]
pub struct CommentSavedInsertForm {
    pub comment_id: Uuid,
    pub user_id: Uuid,
}

// ============================================================
// post_hidden
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = post_hidden)]
pub struct PostHidden {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = post_hidden)]
pub struct PostHiddenInsertForm {
    pub post_id: Uuid,
    pub user_id: Uuid,
}

// ============================================================
// user_follows
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_follows)]
pub struct UserFollow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub follower_id: Uuid,
    pub is_pending: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = user_follows)]
pub struct UserFollowInsertForm {
    pub user_id: Uuid,
    pub follower_id: Uuid,
    pub is_pending: bool,
}

// ============================================================
// board_languages (i32 PK, i32 language_id)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = board_languages)]
pub struct BoardLanguage {
    pub id: i32,
    pub board_id: Uuid,
    pub language_id: i32,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = board_languages)]
pub struct BoardLanguageInsertForm {
    pub board_id: Uuid,
    pub language_id: i32,
}

// ============================================================
// site_languages (i32 PK, i32 language_id)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = site_languages)]
pub struct SiteLanguage {
    pub id: i32,
    pub site_id: Uuid,
    pub language_id: i32,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = site_languages)]
pub struct SiteLanguageInsertForm {
    pub site_id: Uuid,
    pub language_id: i32,
}

// ============================================================
// user_languages (i32 PK, i32 language_id)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_languages)]
pub struct UserLanguage {
    pub id: i32,
    pub user_id: Uuid,
    pub language_id: i32,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = user_languages)]
pub struct UserLanguageInsertForm {
    pub user_id: Uuid,
    pub language_id: i32,
}
