use crate::schema::{rate_limits, registration_applications, site_invites};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// registration_applications
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = registration_applications)]
pub struct RegistrationApplication {
    pub id: Uuid,
    pub user_id: Uuid,
    pub answer: String,
    pub admin_id: Option<Uuid>,
    pub deny_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = registration_applications)]
pub struct RegistrationApplicationInsertForm {
    pub user_id: Uuid,
    pub answer: String,
}

// ============================================================
// site_invites
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = site_invites)]
pub struct SiteInvite {
    pub id: Uuid,
    pub verification_code: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = site_invites)]
pub struct SiteInviteInsertForm {
    pub verification_code: String,
}

// ============================================================
// rate_limits
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = rate_limits)]
pub struct RateLimit {
    pub id: Uuid,
    pub site_id: Uuid,
    pub message: i32,
    pub message_per_second: i32,
    pub post: i32,
    pub post_per_second: i32,
    pub register: i32,
    pub register_per_second: i32,
    pub image: i32,
    pub image_per_second: i32,
    pub comment: i32,
    pub comment_per_second: i32,
    pub search: i32,
    pub search_per_second: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = rate_limits)]
pub struct RateLimitInsertForm {
    pub site_id: Uuid,
    pub message: i32,
    pub message_per_second: i32,
    pub post: i32,
    pub post_per_second: i32,
    pub register: i32,
    pub register_per_second: i32,
    pub image: i32,
    pub image_per_second: i32,
    pub comment: i32,
    pub comment_per_second: i32,
    pub search: i32,
    pub search_per_second: i32,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = rate_limits)]
pub struct RateLimitUpdateForm {
    pub message: Option<i32>,
    pub message_per_second: Option<i32>,
    pub post: Option<i32>,
    pub post_per_second: Option<i32>,
    pub register: Option<i32>,
    pub register_per_second: Option<i32>,
    pub image: Option<i32>,
    pub image_per_second: Option<i32>,
    pub comment: Option<i32>,
    pub comment_per_second: Option<i32>,
    pub search: Option<i32>,
    pub search_per_second: Option<i32>,
}
