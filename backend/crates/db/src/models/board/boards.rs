use crate::enums::*;
use crate::schema::boards;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A board (community / sub-forum) that users can post in.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = boards)]
pub struct Board {
    pub id: Uuid,
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub sidebar: Option<String>,
    pub sidebar_html: Option<String>,
    pub icon: Option<String>,
    pub banner: Option<String>,
    pub primary_color: String,
    pub secondary_color: String,
    pub hover_color: String,
    pub is_nsfw: bool,
    pub is_hidden: bool,
    pub is_removed: bool,
    pub is_banned: bool,
    pub is_posting_restricted_to_mods: bool,
    pub exclude_from_all: bool,
    pub ban_reason: Option<String>,
    pub public_ban_reason: Option<String>,
    pub banned_by: Option<Uuid>,
    pub banned_at: Option<DateTime<Utc>>,
    pub section_config: i32,
    pub section_order: Option<String>,
    pub default_section: Option<String>,
    pub wiki_enabled: bool,
    pub wiki_require_approval: Option<bool>,
    pub wiki_default_view_permission: DbWikiPermission,
    pub wiki_default_edit_permission: DbWikiPermission,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Form for inserting a new board.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = boards)]
pub struct BoardInsertForm {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub sidebar: Option<String>,
    pub sidebar_html: Option<String>,
    pub icon: Option<String>,
    pub banner: Option<String>,
    pub primary_color: String,
    pub secondary_color: String,
    pub hover_color: String,
    pub is_nsfw: bool,
    pub is_hidden: bool,
    pub is_removed: bool,
    pub is_banned: bool,
    pub is_posting_restricted_to_mods: bool,
    pub exclude_from_all: bool,
    pub ban_reason: Option<String>,
    pub public_ban_reason: Option<String>,
    pub banned_by: Option<Uuid>,
    pub banned_at: Option<DateTime<Utc>>,
    pub section_config: i32,
    pub section_order: Option<String>,
    pub default_section: Option<String>,
    pub wiki_enabled: bool,
    pub wiki_require_approval: Option<bool>,
    pub wiki_default_view_permission: DbWikiPermission,
    pub wiki_default_edit_permission: DbWikiPermission,
}

/// Form for updating an existing board. All fields optional so only
/// changed columns are included in the UPDATE.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = boards)]
pub struct BoardUpdateForm {
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub sidebar: Option<Option<String>>,
    pub sidebar_html: Option<Option<String>>,
    pub icon: Option<Option<String>>,
    pub banner: Option<Option<String>>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub hover_color: Option<String>,
    pub is_nsfw: Option<bool>,
    pub is_hidden: Option<bool>,
    pub is_removed: Option<bool>,
    pub is_banned: Option<bool>,
    pub is_posting_restricted_to_mods: Option<bool>,
    pub exclude_from_all: Option<bool>,
    pub ban_reason: Option<Option<String>>,
    pub public_ban_reason: Option<Option<String>>,
    pub banned_by: Option<Option<Uuid>>,
    pub banned_at: Option<Option<DateTime<Utc>>>,
    pub section_config: Option<i32>,
    pub section_order: Option<Option<String>>,
    pub default_section: Option<Option<String>>,
    pub wiki_enabled: Option<bool>,
    pub wiki_require_approval: Option<Option<bool>>,
    pub wiki_default_view_permission: Option<DbWikiPermission>,
    pub wiki_default_edit_permission: Option<DbWikiPermission>,
    pub deleted_at: Option<Option<DateTime<Utc>>>,
}
