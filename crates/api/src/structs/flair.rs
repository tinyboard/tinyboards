use async_graphql::*;
use serde::{Deserialize, Serialize};

// ===== GraphQL Types =====

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct FlairTemplate {
    pub id: i32,
    pub board_id: Option<i32>,
    pub flair_type: String,
    pub text_display: String,
    pub text_editable: bool,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub style_config: Option<String>, // JSON string
    pub emoji_ids: Option<Vec<i32>>,
    pub max_text_length: Option<i32>,
    pub category: Option<String>,
    pub display_order: i32,
    pub requires_approval: bool,
    pub is_active: bool,
    pub usage_count: i32,
    pub creation_date: String,
    pub updated: Option<String>,
    pub created_by: i32,
}

#[ComplexObject]
impl FlairTemplate {
    /// Parse and return the style configuration as a FlairStyle object
    async fn style(&self) -> Option<FlairStyle> {
        self.style_config.as_ref().and_then(|s| {
            serde_json::from_str::<FlairStyle>(s).ok()
        })
    }
}

#[derive(SimpleObject, Clone, Serialize, Deserialize)]
pub struct FlairStyle {
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
    pub border_width: Option<i32>,
    pub border_radius: Option<i32>,
    pub font_weight: Option<String>,
    pub font_size: Option<String>,
    pub padding: Option<String>,
    pub margin: Option<String>,
    pub custom_css: Option<String>,
}

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct PostFlair {
    pub id: i32,
    pub post_id: i32,
    pub template_id: Option<i32>,
    pub text_display: String,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub style_config: Option<String>,
    pub emoji_ids: Option<Vec<i32>>,
    pub creation_date: String,
    pub assigned_by: i32,
}

#[ComplexObject]
impl PostFlair {
    async fn style(&self) -> Option<FlairStyle> {
        self.style_config.as_ref().and_then(|s| {
            serde_json::from_str::<FlairStyle>(s).ok()
        })
    }
}

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct UserFlair {
    pub id: i32,
    pub user_id: i32,
    pub board_id: Option<i32>,
    pub template_id: Option<i32>,
    pub text_display: String,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub style_config: Option<String>,
    pub emoji_ids: Option<Vec<i32>>,
    pub is_approved: bool,
    pub creation_date: String,
    pub assigned_by: Option<i32>,
    pub approved_at: Option<String>,
    pub approved_by: Option<i32>,
}

#[ComplexObject]
impl UserFlair {
    async fn style(&self) -> Option<FlairStyle> {
        self.style_config.as_ref().and_then(|s| {
            serde_json::from_str::<FlairStyle>(s).ok()
        })
    }
}

#[derive(SimpleObject, Clone)]
pub struct FlairAggregates {
    pub template_id: i32,
    pub usage_count: i64,
    pub post_count: i64,
    pub user_count: i64,
}

#[derive(SimpleObject, Clone)]
pub struct FlairCategory {
    pub name: String,
    pub display_order: i32,
    pub template_count: i32,
}

// ===== Input Types =====

#[derive(InputObject)]
pub struct CreateFlairTemplateInput {
    pub board_id: Option<i32>,
    pub flair_type: FlairType,
    pub text_display: String,
    pub text_editable: Option<bool>,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub style_config: Option<FlairStyleInput>,
    pub emoji_ids: Option<Vec<i32>>,
    pub max_text_length: Option<i32>,
    pub category: Option<String>,
    pub display_order: Option<i32>,
    pub requires_approval: Option<bool>,
}

#[derive(InputObject)]
pub struct UpdateFlairTemplateInput {
    pub text_display: Option<String>,
    pub text_editable: Option<bool>,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub style_config: Option<FlairStyleInput>,
    pub emoji_ids: Option<Vec<i32>>,
    pub max_text_length: Option<i32>,
    pub category: Option<String>,
    pub display_order: Option<i32>,
    pub requires_approval: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(InputObject)]
pub struct AssignPostFlairInput {
    pub post_id: i32,
    pub template_id: Option<i32>,
    pub text_display: String,
    pub emoji_ids: Option<Vec<i32>>,
}

#[derive(InputObject)]
pub struct AssignUserFlairInput {
    pub user_id: i32,
    pub board_id: Option<i32>,
    pub template_id: Option<i32>,
    pub text_display: String,
    pub emoji_ids: Option<Vec<i32>>,
}

#[derive(InputObject, Serialize, Deserialize)]
pub struct FlairStyleInput {
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
    pub border_width: Option<i32>,
    pub border_radius: Option<i32>,
    pub font_weight: Option<String>,
    pub font_size: Option<String>,
    pub padding: Option<String>,
    pub margin: Option<String>,
    pub custom_css: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateFlairFiltersInput {
    pub board_id: Option<i32>,
    pub hidden_flair_ids: Option<Vec<i32>>,
    pub highlighted_flair_ids: Option<Vec<i32>>,
}

// ===== Enums =====

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum FlairType {
    #[graphql(name = "post")]
    Post,
    #[graphql(name = "user")]
    User,
}

impl FlairType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FlairType::Post => "post",
            FlairType::User => "user",
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum FlairScope {
    #[graphql(name = "site")]
    Site,
    #[graphql(name = "board")]
    Board,
}

impl FlairScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            FlairScope::Site => "site",
            FlairScope::Board => "board",
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ApprovalStatus {
    #[graphql(name = "pending")]
    Pending,
    #[graphql(name = "approved")]
    Approved,
    #[graphql(name = "rejected")]
    Rejected,
    #[graphql(name = "auto_approved")]
    AutoApproved,
}

impl ApprovalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApprovalStatus::Pending => "pending",
            ApprovalStatus::Approved => "approved",
            ApprovalStatus::Rejected => "rejected",
            ApprovalStatus::AutoApproved => "auto_approved",
        }
    }
}

// ===== Helper conversions =====

impl FlairStyleInput {
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| Error::new(format!("Failed to serialize style config: {}", e)))
    }
}

impl From<FlairStyle> for FlairStyleInput {
    fn from(style: FlairStyle) -> Self {
        Self {
            background_color: style.background_color,
            text_color: style.text_color,
            border_color: style.border_color,
            border_width: style.border_width,
            border_radius: style.border_radius,
            font_weight: style.font_weight,
            font_size: style.font_size,
            padding: style.padding,
            margin: style.margin,
            custom_css: style.custom_css,
        }
    }
}

// ===== Database to GraphQL Type Conversions =====

impl From<tinyboards_db::models::flair::FlairTemplate> for FlairTemplate {
    fn from(db: tinyboards_db::models::flair::FlairTemplate) -> Self {
        Self {
            id: db.id,
            board_id: Some(db.board_id),
            flair_type: db.flair_type,
            text_display: db.text_display,
            text_editable: db.is_editable,
            background_color: Some(db.background_color),
            text_color: Some(db.text_color),
            style_config: Some(db.style_config.to_string()),
            emoji_ids: Some(db.emoji_ids.into_iter().filter_map(|id| id).collect()),
            max_text_length: Some(db.max_text_length),
            category: None,  // Not in current schema
            display_order: db.display_order,
            requires_approval: db.requires_approval,
            is_active: db.is_active,
            usage_count: db.usage_count,
            creation_date: db.creation_date.to_string(),
            updated: Some(db.updated.to_string()),
            created_by: db.created_by,
        }
    }
}

impl From<tinyboards_db::models::flair::PostFlair> for PostFlair {
    fn from(db: tinyboards_db::models::flair::PostFlair) -> Self {
        Self {
            id: db.id,
            post_id: db.post_id,
            template_id: Some(db.flair_template_id),
            text_display: db.custom_text.unwrap_or_default(),
            background_color: db.custom_background_color,
            text_color: db.custom_text_color,
            style_config: None,
            emoji_ids: None,
            creation_date: db.creation_date.to_string(),
            assigned_by: db.assigned_by,
        }
    }
}

impl From<tinyboards_db::models::flair::UserFlair> for UserFlair {
    fn from(db: tinyboards_db::models::flair::UserFlair) -> Self {
        Self {
            id: db.id,
            user_id: db.user_id,
            board_id: Some(db.board_id),
            template_id: Some(db.flair_template_id),
            text_display: db.custom_text.unwrap_or_default(),
            background_color: db.custom_background_color,
            text_color: db.custom_text_color,
            style_config: None,
            emoji_ids: None,
            is_approved: db.is_approved,
            creation_date: db.creation_date.to_string(),
            assigned_by: None,  // Not in current schema
            approved_at: db.approved_at.map(|d| d.to_string()),
            approved_by: db.approved_by,
        }
    }
}
