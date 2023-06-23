use crate::schema::comment_report;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = comment_report)]
pub struct CommentReport {
    pub id: i32,
    pub creator_id: i32,
    pub comment_id: i32,
    pub original_comment_text: String,
    pub reason: String,
    pub resolved: bool,
    pub resolver_id: Option<i32>,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = comment_report)]
pub struct CommentReportForm {
    pub creator_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub original_comment_text: Option<String>,
    pub reason: Option<String>,
    pub resolved: Option<bool>,
    pub resolver_id: Option<i32>,
    pub creation_date: Option<NaiveDateTime>,
    pub updated: Option<NaiveDateTime>,
}
