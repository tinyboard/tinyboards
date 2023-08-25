use crate::sensitive::Sensitive;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tinyboards_db::{newtypes::DbUrl};
use tinyboards_db_views::structs::{LocalUserSettingsView, PersonMentionView, CommentReplyView, LoggedInUserView, PersonView, CommentView, PostView, BoardModeratorView};

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
    pub user: LoggedInUserView,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileResponse {
    pub username: String,
    pub bio: String,
    pub id: i32,
    pub avatar_url: Option<DbUrl>,
    pub banner_url: Option<DbUrl>,
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
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
    pub display_name: Option<String>,
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
pub struct GetPersonMentions {
    pub sort: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub unread_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetPersonMentionsResponse {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
/// ban a person from the site.
pub struct BanPerson {
    pub person_id: i32,
    pub ban: bool,
    pub remove_data: Option<bool>,
    pub reason: Option<String>,
    pub expires: Option<i64>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
/// A response for a banned person.
pub struct BanPersonResponse {
  pub person_view: PersonView,
  pub banned: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Delete your account.
pub struct DeleteAccount {
  pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response of deleting your account.
pub struct DeleteAccountResponse {}


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Gets a person's details.
///
/// Either person_id, or username are required.
pub struct GetPersonDetails {
  pub person_id: Option<i32>,
  /// Example: kroner , or kroner@xyz.tld
  pub username: Option<String>,
  pub sort: Option<String>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub board_id: Option<i32>,
  pub saved_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// A person's details response.
pub struct GetPersonDetailsResponse {
  pub person_view: PersonView,
  pub comments: Vec<CommentView>,
  pub posts: Vec<PostView>,
  pub moderates: Vec<BoardModeratorView>,
}