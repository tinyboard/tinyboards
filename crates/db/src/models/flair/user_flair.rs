use crate::schema::user_flairs;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = user_flairs)]
pub struct UserFlair {
    pub id: i32,
    pub user_id: i32,
    pub board_id: i32,
    pub flair_template_id: i32,
    pub custom_text: Option<String>,
    pub custom_text_color: Option<String>,
    pub custom_background_color: Option<String>,
    pub is_approved: bool,
    pub approved_at: Option<NaiveDateTime>,
    pub approved_by: Option<i32>,
    pub creation_date: NaiveDateTime,
    pub is_self_assigned: bool,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = user_flairs)]
pub struct UserFlairForm {
    pub user_id: Option<i32>,
    pub board_id: Option<i32>,
    pub flair_template_id: Option<i32>,
    pub custom_text: Option<Option<String>>,
    pub custom_text_color: Option<Option<String>>,
    pub custom_background_color: Option<Option<String>>,
    pub is_approved: Option<bool>,
    pub approved_by: Option<Option<i32>>,
    pub is_self_assigned: Option<bool>,
}
