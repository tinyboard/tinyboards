use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{schema::post_report, newtypes::DbUrl};
use chrono::NaiveDateTime;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = post_report)]
pub struct PostReport {
    pub id: i32,
    pub creator_id: i32,
    pub post_id: i32,
    pub original_post_title: String,
    pub original_post_url: Option<DbUrl>,
    pub original_post_body: Option<String>,
    pub reason: String,
    pub resolved: bool,
    pub resolver_id: Option<i32>,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Clone, Insertable, AsChangeset, Default)]
#[diesel(table_name = post_report)]
pub struct PostReportForm {
    pub creator_id: Option<i32>,
    pub post_id: Option<i32>,
    pub original_post_title: Option<String>,
    pub original_post_url: Option<DbUrl>,
    pub original_post_body: Option<String>,
    pub reason: Option<String>,
    pub resolved: Option<bool>,
    pub resolver_id: Option<i32>,
    pub creation_date: Option<NaiveDateTime>,
    pub updated: Option<NaiveDateTime>,
}