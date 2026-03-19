use crate::enums::DbReportStatus;
use crate::schema::post_reports;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Queryable struct for the post_reports table.
/// Field order matches schema.rs column order exactly.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = post_reports)]
pub struct PostReport {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub post_id: Uuid,
    pub original_post_title: String,
    pub original_post_url: Option<String>,
    pub original_post_body: Option<String>,
    pub reason: String,
    pub status: DbReportStatus,
    pub resolver_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Insert form for creating a new post report.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = post_reports)]
pub struct PostReportInsertForm {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub post_id: Uuid,
    pub original_post_title: String,
    pub original_post_url: Option<String>,
    pub original_post_body: Option<String>,
    pub reason: String,
    pub status: DbReportStatus,
}

/// Update form for modifying an existing post report (resolution).
#[derive(Debug, Clone, AsChangeset, Default)]
#[diesel(table_name = post_reports)]
pub struct PostReportUpdateForm {
    pub status: Option<DbReportStatus>,
    pub resolver_id: Option<Option<Uuid>>,
    pub updated_at: Option<DateTime<Utc>>,
}
