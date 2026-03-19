use async_graphql::*;
use serde::{Deserialize, Serialize};
use tinyboards_db::models::flair::{
    FlairCategory as DbFlairCategory, FlairTemplate as DbFlairTemplate, PostFlair as DbPostFlair,
    UserFlair as DbUserFlair,
};

// ===== GraphQL Types =====

#[derive(SimpleObject, Clone)]
pub struct FlairTemplate {
    pub id: ID,
    pub board_id: ID,
    pub flair_type: String,
    pub template_name: String,
    pub template_key: Option<String>,
    pub text_display: String,
    pub text_color: String,
    pub background_color: String,
    pub style_config: String, // JSON string
    pub emoji_ids: Vec<Option<i32>>,
    pub is_mod_only: bool,
    pub is_editable: bool,
    pub max_emoji_count: i32,
    pub max_text_length: i32,
    pub is_requires_approval: bool,
    pub display_order: i32,
    pub is_active: bool,
    pub usage_count: i32,
    pub category_id: Option<ID>,
    pub created_by: ID,
    #[graphql(name = "createdAt")]
    pub created_at: String,
    #[graphql(name = "updatedAt")]
    pub updated_at: String,
}

impl From<DbFlairTemplate> for FlairTemplate {
    fn from(db: DbFlairTemplate) -> Self {
        let flair_type_str = match db.flair_type {
            tinyboards_db::enums::DbFlairType::Post => "post",
            tinyboards_db::enums::DbFlairType::User => "user",
        };
        Self {
            id: db.id.to_string().into(),
            board_id: db.board_id.to_string().into(),
            flair_type: flair_type_str.to_string(),
            template_name: db.template_name,
            template_key: db.template_key,
            text_display: db.text_display,
            text_color: db.text_color,
            background_color: db.background_color,
            style_config: db.style_config.to_string(),
            emoji_ids: db.emoji_ids,
            is_mod_only: db.is_mod_only,
            is_editable: db.is_editable,
            max_emoji_count: db.max_emoji_count,
            max_text_length: db.max_text_length,
            is_requires_approval: db.is_requires_approval,
            display_order: db.display_order,
            is_active: db.is_active,
            usage_count: db.usage_count,
            category_id: db.category_id.map(|id| id.to_string().into()),
            created_by: db.created_by.to_string().into(),
            created_at: db.created_at.to_string(),
            updated_at: db.updated_at.to_string(),
        }
    }
}

#[derive(SimpleObject, Clone, Serialize, Deserialize)]
pub struct FlairStyle {
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
    pub border_width: Option<i32>,
    pub border_radius: Option<i32>,
    pub border_style: Option<String>,
    pub font_weight: Option<String>,
    pub font_size: Option<String>,
    pub padding: Option<String>,
    pub margin: Option<String>,
    pub custom_css: Option<String>,
    pub shadow_color: Option<String>,
    pub shadow_offset_x: Option<i32>,
    pub shadow_offset_y: Option<i32>,
    pub shadow_blur: Option<i32>,
    pub animation_type: Option<String>,
    pub animation_duration: Option<i32>,
    pub gradient_start: Option<String>,
    pub gradient_end: Option<String>,
    pub gradient_direction: Option<String>,
}

#[derive(SimpleObject, Clone)]
pub struct PostFlair {
    pub id: ID,
    pub post_id: ID,
    pub flair_template_id: ID,
    pub custom_text: Option<String>,
    pub custom_text_color: Option<String>,
    pub custom_background_color: Option<String>,
    pub assigned_by: ID,
    pub is_original_author: bool,
    #[graphql(name = "createdAt")]
    pub created_at: String,
}

impl From<DbPostFlair> for PostFlair {
    fn from(db: DbPostFlair) -> Self {
        Self {
            id: db.id.to_string().into(),
            post_id: db.post_id.to_string().into(),
            flair_template_id: db.flair_template_id.to_string().into(),
            custom_text: db.custom_text,
            custom_text_color: db.custom_text_color,
            custom_background_color: db.custom_background_color,
            assigned_by: db.assigned_by.to_string().into(),
            is_original_author: db.is_original_author,
            created_at: db.created_at.to_string(),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct UserFlair {
    pub id: ID,
    pub user_id: ID,
    pub board_id: ID,
    pub flair_template_id: ID,
    pub custom_text: Option<String>,
    pub custom_text_color: Option<String>,
    pub custom_background_color: Option<String>,
    pub is_approved: bool,
    pub approved_at: Option<String>,
    pub approved_by: Option<ID>,
    pub is_self_assigned: bool,
    #[graphql(name = "createdAt")]
    pub created_at: String,
}

impl From<DbUserFlair> for UserFlair {
    fn from(db: DbUserFlair) -> Self {
        Self {
            id: db.id.to_string().into(),
            user_id: db.user_id.to_string().into(),
            board_id: db.board_id.to_string().into(),
            flair_template_id: db.flair_template_id.to_string().into(),
            custom_text: db.custom_text,
            custom_text_color: db.custom_text_color,
            custom_background_color: db.custom_background_color,
            is_approved: db.is_approved,
            approved_at: db.approved_at.map(|d| d.to_string()),
            approved_by: db.approved_by.map(|id| id.to_string().into()),
            is_self_assigned: db.is_self_assigned,
            created_at: db.created_at.to_string(),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct FlairAggregatesView {
    pub flair_template_id: ID,
    pub total_usage_count: i32,
    pub post_usage_count: i32,
    pub user_usage_count: i32,
    pub active_user_count: i32,
    pub usage_last_day: i32,
    pub usage_last_week: i32,
    pub usage_last_month: i32,
    pub total_post_comments: i32,
    pub total_post_score: i32,
    pub last_used_at: Option<String>,
}

#[derive(SimpleObject, Clone)]
pub struct FlairCategory {
    pub id: ID,
    pub board_id: ID,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub display_order: i32,
    pub created_by: ID,
    #[graphql(name = "createdAt")]
    pub created_at: String,
    #[graphql(name = "updatedAt")]
    pub updated_at: String,
}

impl From<DbFlairCategory> for FlairCategory {
    fn from(db: DbFlairCategory) -> Self {
        Self {
            id: db.id.to_string().into(),
            board_id: db.board_id.to_string().into(),
            name: db.name,
            description: db.description,
            color: db.color,
            display_order: db.display_order,
            created_by: db.created_by.to_string().into(),
            created_at: db.created_at.to_string(),
            updated_at: db.updated_at.to_string(),
        }
    }
}

// ===== Input Types =====

#[derive(InputObject)]
pub struct CreateFlairCategoryInput {
    pub board_id: ID,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub display_order: Option<i32>,
}

#[derive(InputObject)]
pub struct UpdateFlairCategoryInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub display_order: Option<i32>,
}

#[derive(InputObject)]
pub struct CreateFlairTemplateInput {
    pub board_id: ID,
    pub flair_type: FlairType,
    pub template_name: String,
    pub text_display: String,
    pub text_color: Option<String>,
    pub background_color: Option<String>,
    pub style_config: Option<FlairStyleInput>,
    pub emoji_ids: Option<Vec<i32>>,
    pub max_emoji_count: Option<i32>,
    pub max_text_length: Option<i32>,
    pub category_id: Option<ID>,
    pub display_order: Option<i32>,
    pub is_mod_only: Option<bool>,
    pub is_editable: Option<bool>,
    pub is_requires_approval: Option<bool>,
}

#[derive(InputObject)]
pub struct UpdateFlairTemplateInput {
    pub template_name: Option<String>,
    pub text_display: Option<String>,
    pub text_color: Option<String>,
    pub background_color: Option<String>,
    pub style_config: Option<FlairStyleInput>,
    pub emoji_ids: Option<Vec<i32>>,
    pub max_emoji_count: Option<i32>,
    pub max_text_length: Option<i32>,
    pub category_id: Option<ID>,
    pub display_order: Option<i32>,
    pub is_mod_only: Option<bool>,
    pub is_editable: Option<bool>,
    pub is_requires_approval: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(InputObject)]
pub struct AssignPostFlairInput {
    pub post_id: ID,
    pub flair_template_id: ID,
    pub custom_text: Option<String>,
    pub custom_text_color: Option<String>,
    pub custom_background_color: Option<String>,
}

#[derive(InputObject)]
pub struct AssignUserFlairInput {
    pub user_id: ID,
    pub board_id: ID,
    pub flair_template_id: ID,
    pub custom_text: Option<String>,
    pub custom_text_color: Option<String>,
    pub custom_background_color: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateFlairFiltersInput {
    pub board_id: ID,
    pub filter_mode: Option<FilterMode>,
    pub included_flair_ids: Option<Vec<i32>>,
    pub excluded_flair_ids: Option<Vec<i32>>,
}

#[derive(InputObject, Serialize, Deserialize)]
pub struct FlairStyleInput {
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
    pub border_width: Option<i32>,
    pub border_radius: Option<i32>,
    pub border_style: Option<String>,
    pub font_weight: Option<String>,
    pub font_size: Option<String>,
    pub padding: Option<String>,
    pub margin: Option<String>,
    pub custom_css: Option<String>,
    pub shadow_color: Option<String>,
    pub shadow_offset_x: Option<i32>,
    pub shadow_offset_y: Option<i32>,
    pub shadow_blur: Option<i32>,
    pub animation_type: Option<String>,
    pub animation_duration: Option<i32>,
    pub gradient_start: Option<String>,
    pub gradient_end: Option<String>,
    pub gradient_direction: Option<String>,
}

// ===== Enums =====

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum FlairType {
    #[graphql(name = "post")]
    Post,
    #[graphql(name = "user")]
    User,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum FilterMode {
    #[graphql(name = "include")]
    Include,
    #[graphql(name = "exclude")]
    Exclude,
}

// ===== Helper conversions =====

impl FlairStyleInput {
    pub fn to_json_value(&self) -> Result<serde_json::Value> {
        serde_json::to_value(self)
            .map_err(|e| Error::new(format!("Failed to serialize style config: {}", e)))
    }
}
