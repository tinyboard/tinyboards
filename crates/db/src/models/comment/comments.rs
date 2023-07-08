use crate::{newtypes::DbUrl, schema::comments};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = comments)]
pub struct Comment {
    pub id: i32,
    pub creator_id: i32,
    pub post_id: i32,
    pub parent_id: Option<i32>,
    pub body: String,
    pub body_html: String,
    pub is_removed: bool,
    pub read: bool,
    pub creation_date: NaiveDateTime,
    pub level: i32,
    pub is_deleted: bool,
    pub updated: Option<NaiveDateTime>,
    pub is_locked: bool,
    pub board_id: i32,
    pub local: bool,
    pub ap_id: Option<DbUrl>,
    pub language_id: i32,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = comments)]
pub struct CommentForm {
    pub creator_id: Option<i32>,
    pub post_id: Option<i32>,
    pub parent_id: Option<i32>,
    pub body: Option<String>,
    pub body_html: Option<String>,
    pub is_removed: Option<bool>,
    pub read: Option<bool>,
    pub level: Option<i32>,
    pub updated: Option<NaiveDateTime>,
    pub is_deleted: Option<bool>,
    pub board_id: Option<i32>,
    pub local: Option<bool>,
    pub ap_id: Option<DbUrl>,
    pub language_id: Option<i32>,
    pub creation_date: Option<NaiveDateTime>,
}
