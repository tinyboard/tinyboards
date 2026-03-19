use chrono::NaiveDateTime;
use diesel::sql_types::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// Database row types (for sql_query + QueryableByName)
// ============================================================

/// Minimal user data needed for auth operations.
/// Loaded via raw SQL to decouple from tinyboards_db model layer.
#[derive(Debug, Clone, diesel::QueryableByName)]
pub struct AuthUser {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub email: Option<String>,
    #[diesel(sql_type = Text)]
    pub passhash: String,
    #[diesel(sql_type = Bool)]
    pub is_email_verified: bool,
    #[diesel(sql_type = Bool)]
    pub is_banned: bool,
    #[diesel(sql_type = Bool)]
    pub is_admin: bool,
    #[diesel(sql_type = Integer)]
    pub admin_level: i32,
    #[diesel(sql_type = Bool)]
    pub is_application_accepted: bool,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub deleted_at: Option<NaiveDateTime>,
}

/// Auth session row from the auth_sessions table.
#[derive(Debug, Clone, diesel::QueryableByName)]
pub struct AuthSessionRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub user_id: Uuid,
    #[diesel(sql_type = Text)]
    pub refresh_token_hash: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub user_agent: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub ip_address: Option<String>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub last_used_at: Option<NaiveDateTime>,
    #[diesel(sql_type = Timestamptz)]
    pub expires_at: NaiveDateTime,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: NaiveDateTime,
}

/// Password reset row from the password_resets table.
#[derive(Debug, Clone, diesel::QueryableByName)]
pub struct PasswordResetRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub user_id: Uuid,
    #[diesel(sql_type = Text)]
    pub reset_token: String,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: NaiveDateTime,
    #[diesel(sql_type = Timestamptz)]
    pub expires_at: NaiveDateTime,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub used_at: Option<NaiveDateTime>,
}

/// Email verification row.
#[derive(Debug, Clone, diesel::QueryableByName)]
pub struct EmailVerificationRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub user_id: Uuid,
    #[diesel(sql_type = Text)]
    pub email: String,
    #[diesel(sql_type = Text)]
    pub verification_code: String,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: NaiveDateTime,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub verified_at: Option<NaiveDateTime>,
}

/// JWT secret row from the secrets table.
#[derive(Debug, Clone, diesel::QueryableByName)]
pub struct JwtSecretRow {
    #[diesel(sql_type = VarChar)]
    pub jwt_secret: String,
}

/// Registration mode from the site table.
#[derive(Debug, Clone, diesel::QueryableByName)]
pub struct SiteRegistrationInfo {
    #[diesel(sql_type = Text)]
    pub registration_mode: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub application_question: Option<String>,
}

/// Result of user creation (just the id and name).
#[derive(Debug, Clone, diesel::QueryableByName)]
pub struct CreatedUser {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = Text)]
    pub name: String,
}

// ============================================================
// API request/response types
// ============================================================

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username_or_email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub password: String,
    pub invite_code: Option<String>,
    pub application_answer: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    // No body needed; the refresh token comes from the cookie.
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetComplete {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct EmailVerifyRequest {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct RequestEmailVerification {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserInfo>,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub name: String,
    pub is_admin: bool,
    pub admin_level: i32,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub account_created: bool,
    pub application_submitted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Authenticated user info attached to request extensions by middleware.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub role: UserRole,
}

/// User role extracted from JWT claims.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    User,
    Admin(i32),
}

impl UserRole {
    pub fn from_admin_fields(is_admin: bool, admin_level: i32) -> Self {
        if is_admin {
            UserRole::Admin(admin_level)
        } else {
            UserRole::User
        }
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin(_))
    }
}
