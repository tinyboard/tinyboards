use crate::sensitive::Sensitive;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tinyboards_db::newtypes::DbUrl;
use tinyboards_db_views::structs::{LocalUserSettingsView, PersonMentionView, CommentReplyView, LocalUserView};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Login {
    pub username_or_email: Sensitive<String>,
    pub password: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignupResponse {
    pub jwt: Option<Sensitive<String>>,
    pub registration_created: bool,
    pub verify_email_sent: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginResponse {
    pub jwt: Sensitive<String>,
    pub user: LocalUserView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Register {
    pub username: String,
    pub password: Sensitive<String>,
    pub invite_token: Option<String>,
    pub email: Option<String>,
    pub captcha_uuid: Option<String>,
    pub captcha_answer: Option<String>,
    pub answer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProfileResponse {
    pub username: String,
    pub bio: String,
    pub id: i32,
    pub avatar_url: DbUrl,
    pub banner_url: DbUrl,
    pub url: String,
    pub html_url: String,
    pub saved_url: String,
    pub posts_url: String,
    pub comments_url: String,
    pub user_type: String,
    pub is_admin: bool,
    pub display_name: String,
    pub rep: i64,
    pub posts_count: i64,
    pub posts_score: i64,
    pub comments_count: i64,
    pub comments_score: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub is_banned: bool,
    pub is_deleted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Profile {}

#[derive(Deserialize)]
pub struct GetLoggedInUser {}

#[derive(Deserialize)]
pub struct GetPostPath {
    pub post_id: i32,
}

#[derive(Deserialize, Clone)]
pub struct GetUserNamePath {
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetUserSettings {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetUserSettingsResponse {
    pub settings: LocalUserSettingsView,
}


/// Struct for saving user settings, update this with any additional settings we need to be able to set
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SaveUserSettings {
    pub show_nsfw: Option<bool>,
    pub theme: Option<String>,
    pub default_sort_type: Option<i16>,
    pub default_listing_type: Option<i16>,
    pub avatar: Option<DbUrl>,
    pub signature: Option<DbUrl>,
    pub banner: Option<DbUrl>,
    pub email: Option<String>,
    pub bio: Option<String>,
}

/// Struct for changing passwords
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChangePassword {
    pub new_password: Option<String>,
    pub new_password_verify: Option<String>,
    pub old_password: Option<String>,
}

/// Struct for verifying email
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct VerifyEmail {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VerifyEmailResponse {}

/// Struct for accepting site invite
pub struct AcceptSiteInvite {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AcceptSiteInviteResponse {}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetUserMentions {
    pub sort: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub unread_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetUserMentionsResponse {
    pub mentions: Vec<PersonMentionView>,
    pub total_count: i64,
    pub unread_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetCommentReplies {
    pub sort: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub unread_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetCommentRepliesResponse {
    pub replies: Vec<CommentReplyView>,
    pub total_count: i64,
    pub unread_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetUnreadCount {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetUnreadCountResponse {
  pub replies: i64,
  pub mentions: i64,
  pub total_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MarkAllMentionsRead {}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MarkAllRepliesRead {}


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UsernameInfo {
    pub name: String,
    pub avatar: Option<DbUrl>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SearchNames {
    pub q: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SearchNamesResponse {
    pub users: Vec<UsernameInfo>,
}