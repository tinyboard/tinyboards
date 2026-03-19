use crate::enums::DbReportStatus;
use crate::schema::comment_reports;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Queryable struct for the comment_reports table.
/// Field order matches schema.rs column order exactly.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = comment_reports)]
pub struct CommentReport {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub comment_id: Uuid,
    pub original_comment_text: String,
    pub reason: String,
    pub status: DbReportStatus,
    pub resolver_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Insert form for creating a new comment report.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = comment_reports)]
pub struct CommentReportInsertForm {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub comment_id: Uuid,
    pub original_comment_text: String,
    pub reason: String,
    pub status: DbReportStatus,
}

/// Update form for modifying an existing comment report (resolution).
#[derive(Debug, Clone, AsChangeset, Default)]
#[diesel(table_name = comment_reports)]
pub struct CommentReportUpdateForm {
    pub status: Option<DbReportStatus>,
    pub resolver_id: Option<Option<Uuid>>,
    pub updated_at: Option<DateTime<Utc>>,
}
