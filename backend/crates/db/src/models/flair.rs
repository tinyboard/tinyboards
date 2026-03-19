use crate::enums::{DbFlairType, DbFilterMode};
use crate::schema::{
    flair_categories, flair_templates, post_flairs, user_flairs, user_flair_filters,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// flair_categories
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = flair_categories)]
pub struct FlairCategory {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub display_order: i32,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = flair_categories)]
pub struct FlairCategoryInsertForm {
    pub board_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub display_order: i32,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = flair_categories)]
pub struct FlairCategoryUpdateForm {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub display_order: Option<i32>,
}

// ============================================================
// flair_templates
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = flair_templates)]
pub struct FlairTemplate {
    pub id: Uuid,
    pub board_id: Uuid,
    pub flair_type: DbFlairType,
    pub template_name: String,
    pub template_key: Option<String>,
    pub text_display: String,
    pub text_color: String,
    pub background_color: String,
    pub style_config: serde_json::Value,
    pub emoji_ids: Vec<Option<i32>>,
    pub is_mod_only: bool,
    pub is_editable: bool,
    pub max_emoji_count: i32,
    pub max_text_length: i32,
    pub is_requires_approval: bool,
    pub display_order: i32,
    pub is_active: bool,
    pub usage_count: i32,
    pub category_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = flair_templates)]
pub struct FlairTemplateInsertForm {
    pub board_id: Uuid,
    pub flair_type: DbFlairType,
    pub template_name: String,
    pub template_key: Option<String>,
    pub text_display: String,
    pub text_color: String,
    pub background_color: String,
    pub style_config: serde_json::Value,
    pub emoji_ids: Vec<Option<i32>>,
    pub is_mod_only: bool,
    pub is_editable: bool,
    pub max_emoji_count: i32,
    pub max_text_length: i32,
    pub is_requires_approval: bool,
    pub display_order: i32,
    pub is_active: bool,
    pub category_id: Option<Uuid>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = flair_templates)]
pub struct FlairTemplateUpdateForm {
    pub flair_type: Option<DbFlairType>,
    pub template_name: Option<String>,
    pub template_key: Option<Option<String>>,
    pub text_display: Option<String>,
    pub text_color: Option<String>,
    pub background_color: Option<String>,
    pub style_config: Option<serde_json::Value>,
    pub emoji_ids: Option<Vec<Option<i32>>>,
    pub is_mod_only: Option<bool>,
    pub is_editable: Option<bool>,
    pub max_emoji_count: Option<i32>,
    pub max_text_length: Option<i32>,
    pub is_requires_approval: Option<bool>,
    pub display_order: Option<i32>,
    pub is_active: Option<bool>,
    pub category_id: Option<Option<Uuid>>,
}

// ============================================================
// post_flairs
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = post_flairs)]
pub struct PostFlair {
    pub id: Uuid,
    pub post_id: Uuid,
    pub flair_template_id: Uuid,
    pub custom_text: Option<String>,
    pub custom_text_color: Option<String>,
    pub custom_background_color: Option<String>,
    pub assigned_by: Uuid,
    pub is_original_author: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = post_flairs)]
pub struct PostFlairInsertForm {
    pub post_id: Uuid,
    pub flair_template_id: Uuid,
    pub custom_text: Option<String>,
    pub custom_text_color: Option<String>,
    pub custom_background_color: Option<String>,
    pub assigned_by: Uuid,
    pub is_original_author: bool,
}

// ============================================================
// user_flairs
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_flairs)]
pub struct UserFlair {
    pub id: Uuid,
    pub user_id: Uuid,
    pub board_id: Uuid,
    pub flair_template_id: Uuid,
    pub custom_text: Option<String>,
    pub custom_text_color: Option<String>,
    pub custom_background_color: Option<String>,
    pub is_approved: bool,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub is_self_assigned: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = user_flairs)]
pub struct UserFlairInsertForm {
    pub user_id: Uuid,
    pub board_id: Uuid,
    pub flair_template_id: Uuid,
    pub custom_text: Option<String>,
    pub custom_text_color: Option<String>,
    pub custom_background_color: Option<String>,
    pub is_approved: bool,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub is_self_assigned: bool,
}

// ============================================================
// user_flair_filters
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_flair_filters)]
pub struct UserFlairFilter {
    pub id: Uuid,
    pub user_id: Uuid,
    pub board_id: Uuid,
    pub filter_mode: DbFilterMode,
    pub included_flair_ids: Vec<Option<i32>>,
    pub excluded_flair_ids: Vec<Option<i32>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = user_flair_filters)]
pub struct UserFlairFilterInsertForm {
    pub user_id: Uuid,
    pub board_id: Uuid,
    pub filter_mode: DbFilterMode,
    pub included_flair_ids: Vec<Option<i32>>,
    pub excluded_flair_ids: Vec<Option<i32>>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = user_flair_filters)]
pub struct UserFlairFilterUpdateForm {
    pub filter_mode: Option<DbFilterMode>,
    pub included_flair_ids: Option<Vec<Option<i32>>>,
    pub excluded_flair_ids: Option<Vec<Option<i32>>>,
}
