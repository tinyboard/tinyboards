use crate::schema::comment;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = comment)]
pub struct Comment {
    pub id: i32,
    pub creator_id: i32,
    pub post_id: i32,
    pub parent_id: Option<i32>,
    pub body: String,
    pub removed: bool,
    pub read: bool,
    pub published: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub deleted: bool,
    pub level: i32
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = comment)]
pub struct CommentForm {
    pub creator_id: i32,
    pub post_id: i32,
    pub parent_id: Option<i32>,
    pub body: Option<String>,
    pub removed: Option<bool>,
    pub read: Option<bool>,
    pub updated: Option<NaiveDateTime>,
    pub deleted: Option<bool>,
    pub level: i32
}
