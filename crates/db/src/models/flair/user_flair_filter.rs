use crate::schema::user_flair_filters;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    #[strum(ascii_case_insensitive)]
    Hide,
    #[strum(ascii_case_insensitive)]
    Show,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = user_flair_filters)]
pub struct UserFlairFilter {
    pub id: i32,
    pub user_id: i32,
    pub board_id: i32,
    pub filter_mode: String,
    pub included_flair_ids: Vec<Option<i32>>,
    pub excluded_flair_ids: Vec<Option<i32>>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = user_flair_filters)]
pub struct UserFlairFilterForm {
    pub user_id: Option<i32>,
    pub board_id: Option<i32>,
    pub filter_mode: Option<String>,
    pub included_flair_ids: Option<Vec<Option<i32>>>,
    pub excluded_flair_ids: Option<Vec<Option<i32>>>,
}
