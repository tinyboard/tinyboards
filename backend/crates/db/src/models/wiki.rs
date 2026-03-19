use crate::enums::DbWikiPermission;
use crate::schema::{wiki_approved_contributors, wiki_page_revisions, wiki_pages};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// wiki_pages
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = wiki_pages)]
pub struct WikiPage {
    pub id: Uuid,
    pub board_id: Uuid,
    pub slug: String,
    pub title: String,
    pub body: String,
    pub body_html: String,
    pub creator_id: Uuid,
    pub last_edited_by: Option<Uuid>,
    pub view_permission: DbWikiPermission,
    pub edit_permission: DbWikiPermission,
    pub is_locked: bool,
    pub display_order: Option<i32>,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = wiki_pages)]
pub struct WikiPageInsertForm {
    pub board_id: Uuid,
    pub slug: String,
    pub title: String,
    pub body: String,
    pub body_html: String,
    pub creator_id: Uuid,
    pub last_edited_by: Option<Uuid>,
    pub view_permission: DbWikiPermission,
    pub edit_permission: DbWikiPermission,
    pub is_locked: bool,
    pub display_order: Option<i32>,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = wiki_pages)]
pub struct WikiPageUpdateForm {
    pub slug: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub body_html: Option<String>,
    pub last_edited_by: Option<Option<Uuid>>,
    pub view_permission: Option<DbWikiPermission>,
    pub edit_permission: Option<DbWikiPermission>,
    pub is_locked: Option<bool>,
    pub display_order: Option<Option<i32>>,
    pub parent_id: Option<Option<Uuid>>,
    pub deleted_at: Option<Option<DateTime<Utc>>>,
}

// ============================================================
// wiki_page_revisions
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = wiki_page_revisions)]
pub struct WikiPageRevision {
    pub id: Uuid,
    pub page_id: Uuid,
    pub revision_number: i32,
    pub editor_id: Uuid,
    pub edit_summary: Option<String>,
    pub body: String,
    pub body_html: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = wiki_page_revisions)]
pub struct WikiPageRevisionInsertForm {
    pub page_id: Uuid,
    pub revision_number: i32,
    pub editor_id: Uuid,
    pub edit_summary: Option<String>,
    pub body: String,
    pub body_html: String,
}

// ============================================================
// wiki_approved_contributors
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = wiki_approved_contributors)]
pub struct WikiApprovedContributor {
    pub id: Uuid,
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub added_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = wiki_approved_contributors)]
pub struct WikiApprovedContributorInsertForm {
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub added_by: Uuid,
}
