use crate::schema::post_flairs;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = post_flairs)]
pub struct PostFlair {
    pub id: i32,
    pub post_id: i32,
    pub flair_template_id: i32,
    pub custom_text: Option<String>,
    pub custom_text_color: Option<String>,
    pub custom_background_color: Option<String>,
    pub assigned_at: NaiveDateTime,
    pub assigned_by: i32,
    pub is_original_author: bool,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = post_flairs)]
pub struct PostFlairForm {
    pub post_id: Option<i32>,
    pub flair_template_id: Option<i32>,
    pub custom_text: Option<Option<String>>,
    pub custom_text_color: Option<Option<String>>,
    pub custom_background_color: Option<Option<String>>,
    pub assigned_by: Option<i32>,
    pub is_original_author: Option<bool>,
}
