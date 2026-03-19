use crate::schema::{auth_sessions, email_verification, password_resets, secrets};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// auth_sessions
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = auth_sessions)]
pub struct AuthSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token_hash: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = auth_sessions)]
pub struct AuthSessionInsertForm {
    pub user_id: Uuid,
    pub refresh_token_hash: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub expires_at: DateTime<Utc>,
}

// ============================================================
// secrets
// ============================================================

#[derive(Debug, Clone, Queryable, Identifiable)]
#[diesel(table_name = secrets)]
pub struct Secret {
    pub id: Uuid,
    pub jwt_secret: String,
}

// ============================================================
// password_resets
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = password_resets)]
pub struct PasswordReset {
    pub id: Uuid,
    pub user_id: Uuid,
    pub reset_token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = password_resets)]
pub struct PasswordResetInsertForm {
    pub user_id: Uuid,
    pub reset_token: String,
    pub expires_at: DateTime<Utc>,
}

// ============================================================
// email_verification
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = email_verification)]
pub struct EmailVerification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub verification_code: String,
    pub created_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = email_verification)]
pub struct EmailVerificationInsertForm {
    pub user_id: Uuid,
    pub email: String,
    pub verification_code: String,
}
