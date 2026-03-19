use crate::enums::{DbApprovalStatus, DbPostType};
use crate::schema::posts;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Queryable struct for the posts table.
/// Field order matches schema.rs column order exactly.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub post_type: DbPostType,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub body: String,
    pub body_html: String,
    pub image: Option<String>,
    pub alt_text: Option<String>,
    pub slug: String,
    pub creator_id: Uuid,
    pub board_id: Uuid,
    pub language_id: Option<i32>,
    pub is_removed: bool,
    pub is_locked: bool,
    pub is_nsfw: bool,
    pub is_featured_board: bool,
    pub is_featured_local: bool,
    pub approval_status: DbApprovalStatus,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub embed_title: Option<String>,
    pub embed_description: Option<String>,
    pub embed_video_url: Option<String>,
    pub source_url: Option<String>,
    pub last_crawl_date: Option<DateTime<Utc>>,
    pub is_thread: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert form for creating a new post.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = posts)]
pub struct PostInsertForm {
    pub id: Uuid,
    pub title: String,
    pub post_type: DbPostType,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub body: String,
    pub body_html: String,
    pub image: Option<String>,
    pub alt_text: Option<String>,
    pub slug: String,
    pub creator_id: Uuid,
    pub board_id: Uuid,
    pub language_id: Option<i32>,
    pub is_nsfw: bool,
    pub approval_status: DbApprovalStatus,
    pub embed_title: Option<String>,
    pub embed_description: Option<String>,
    pub embed_video_url: Option<String>,
    pub source_url: Option<String>,
    pub is_thread: bool,
}

/// Update form for modifying an existing post.
/// All fields are optional; only set fields will be updated.
#[derive(Debug, Clone, AsChangeset, Default)]
#[diesel(table_name = posts)]
pub struct PostUpdateForm {
    pub title: Option<String>,
    pub post_type: Option<DbPostType>,
    pub url: Option<Option<String>>,
    pub thumbnail_url: Option<Option<String>>,
    pub body: Option<String>,
    pub body_html: Option<String>,
    pub image: Option<Option<String>>,
    pub alt_text: Option<Option<String>>,
    pub slug: Option<String>,
    pub language_id: Option<Option<i32>>,
    pub is_removed: Option<bool>,
    pub is_locked: Option<bool>,
    pub is_nsfw: Option<bool>,
    pub is_featured_board: Option<bool>,
    pub is_featured_local: Option<bool>,
    pub approval_status: Option<DbApprovalStatus>,
    pub approved_by: Option<Option<Uuid>>,
    pub approved_at: Option<Option<DateTime<Utc>>>,
    pub embed_title: Option<Option<String>>,
    pub embed_description: Option<Option<String>>,
    pub embed_video_url: Option<Option<String>>,
    pub source_url: Option<Option<String>>,
    pub last_crawl_date: Option<Option<DateTime<Utc>>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<Option<DateTime<Utc>>>,
}
