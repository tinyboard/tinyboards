use crate::schema::flair_templates;
use crate::utils::DbPool;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = flair_templates)]
pub struct FlairTemplate {
    pub id: i32,
    pub board_id: i32,
    pub flair_type: String,
    pub template_name: String,
    pub template_key: Option<String>,
    pub text_display: String,
    pub text_color: String,
    pub background_color: String,
    pub style_config: serde_json::Value,
    pub emoji_ids: Vec<Option<i32>>,
    pub mod_only: bool,
    pub is_editable: bool,
    pub max_text_length: i32,
    pub requires_approval: bool,
    pub display_order: i32,
    pub is_active: bool,
    pub creation_date: NaiveDateTime,
    pub updated: NaiveDateTime,
    pub created_by: i32,
    pub usage_count: i32,
    pub category_id: Option<i32>,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = flair_templates)]
pub struct FlairTemplateForm {
    pub board_id: Option<i32>,
    pub flair_type: Option<String>,
    pub template_name: Option<String>,
    pub template_key: Option<Option<String>>,
    pub text_display: Option<String>,
    pub text_color: Option<String>,
    pub background_color: Option<String>,
    pub style_config: Option<serde_json::Value>,
    pub emoji_ids: Option<Vec<Option<i32>>>,
    pub mod_only: Option<bool>,
    pub is_editable: Option<bool>,
    pub max_text_length: Option<i32>,
    pub requires_approval: Option<bool>,
    pub display_order: Option<i32>,
    pub is_active: Option<bool>,
    pub category_id: Option<Option<i32>>,
    pub created_by: Option<i32>,
}

impl FlairTemplate {
    pub async fn read(pool: &DbPool, flair_id: i32) -> Result<Self, diesel::result::Error> {
        let conn = &mut pool.get().await.map_err(|_| diesel::result::Error::NotFound)?;
        flair_templates::table
            .find(flair_id)
            .first::<Self>(conn)
            .await
    }

    pub async fn for_board(pool: &DbPool, board_id_param: i32) -> Result<Vec<Self>, diesel::result::Error> {
        let conn = &mut pool.get().await.map_err(|_| diesel::result::Error::NotFound)?;
        flair_templates::table
            .filter(flair_templates::board_id.eq(board_id_param))
            .order(flair_templates::display_order.asc())
            .load::<Self>(conn)
            .await
    }
}
