use crate::enums::DbApprovalStatus;
use crate::schema::comments;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Queryable struct for the comments table.
/// Field order matches schema.rs column order exactly.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = comments)]
pub struct Comment {
    pub id: Uuid,
    pub body: String,
    pub body_html: String,
    pub slug: String,
    pub creator_id: Uuid,
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub board_id: Uuid,
    pub language_id: Option<i32>,
    pub level: i32,
    pub is_removed: bool,
    pub is_locked: bool,
    pub is_read: bool,
    pub is_pinned: bool,
    pub approval_status: DbApprovalStatus,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub quoted_comment_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Insert form for creating a new comment.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = comments)]
pub struct CommentInsertForm {
    pub id: Uuid,
    pub body: String,
    pub body_html: String,
    pub slug: String,
    pub creator_id: Uuid,
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub board_id: Uuid,
    pub language_id: Option<i32>,
    pub level: i32,
    pub approval_status: DbApprovalStatus,
    pub quoted_comment_id: Option<Uuid>,
}

/// Update form for modifying an existing comment.
/// All fields are optional; only set fields will be updated.
#[derive(Debug, Clone, AsChangeset, Default)]
#[diesel(table_name = comments)]
pub struct CommentUpdateForm {
    pub body: Option<String>,
    pub body_html: Option<String>,
    pub language_id: Option<Option<i32>>,
    pub is_removed: Option<bool>,
    pub is_locked: Option<bool>,
    pub is_read: Option<bool>,
    pub is_pinned: Option<bool>,
    pub approval_status: Option<DbApprovalStatus>,
    pub approved_by: Option<Option<Uuid>>,
    pub approved_at: Option<Option<DateTime<Utc>>>,
    pub quoted_comment_id: Option<Option<Uuid>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<Option<DateTime<Utc>>>,
}
