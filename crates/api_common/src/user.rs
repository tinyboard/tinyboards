use crate::sensitive::Sensitive;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tinyboards_db_views::structs::{UserView, UserSettingsView};

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
    pub user: UserView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Register {
    pub username: String,
    pub password: Sensitive<String>,
    // pub password_verify: Sensitive<String>,
    // pub show_nsfw: bool,
    // email = mandatory if email verification enabled on server
    pub email: Option<String>,
    pub captcha_uuid: Option<String>,
    pub captcha_answer: Option<String>,
    // An answer = required if require application is enabled on server
    pub answer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProfileResponse {
    pub username: String,
    pub bio: String,
    pub id: i32,
    pub avatar_url: String,
    pub banner_url: String,
    pub url: String,
    pub html_url: String,
    pub saved_url: String,
    pub posts_url: String,
    pub comments_url: String,
    pub user_type: String,
    pub is_admin: bool,
    pub display_name: String,
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
    pub settings: UserSettingsView,
}