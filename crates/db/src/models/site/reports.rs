use crate::schema::reports;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = reports)]
pub struct Report {
    pub id: i32,
    pub user_id: Option<i32>,
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub reason: Option<String>,
    pub creation_date: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = reports)]
pub struct ReportForm {
    pub user_id: Option<Option<i32>>,
    pub post_id: Option<Option<i32>>,
    pub comment_id: Option<Option<i32>>,
    pub reason: Option<Option<String>>,
}
