use crate::schema::{uploads, content_uploads};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// uploads
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = uploads)]
pub struct Upload {
    pub id: Uuid,
    pub user_id: Uuid,
    pub original_name: String,
    pub file_name: String,
    pub file_path: String,
    pub upload_url: String,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
    pub thumbnail_url: Option<String>,
    pub optimized_url: Option<String>,
    pub processing_status: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = uploads)]
pub struct UploadInsertForm {
    pub user_id: Uuid,
    pub original_name: String,
    pub file_name: String,
    pub file_path: String,
    pub upload_url: String,
    pub size_bytes: i64,
    pub thumbnail_url: Option<String>,
    pub optimized_url: Option<String>,
    pub processing_status: Option<String>,
}

// ============================================================
// content_uploads
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = content_uploads)]
pub struct ContentUpload {
    pub id: Uuid,
    pub upload_id: Uuid,
    pub post_id: Option<Uuid>,
    pub comment_id: Option<Uuid>,
    pub position: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = content_uploads)]
pub struct ContentUploadInsertForm {
    pub upload_id: Uuid,
    pub post_id: Option<Uuid>,
    pub comment_id: Option<Uuid>,
    pub position: Option<i32>,
}
