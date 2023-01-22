use crate::schema::posts;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub type_: String,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub permalink: Option<String>,
    pub body: String,
    pub body_html: String,
    pub creator_id: i32,
    pub board_id: i32,
    pub is_removed: bool,
    pub is_locked: bool,
    pub creation_date: NaiveDateTime,
    pub is_deleted: bool,
    pub is_nsfw: bool,
    pub is_stickied: bool,
    pub updated: Option<NaiveDateTime>,
    pub image: Option<String>,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = posts)]
pub struct PostForm {
    pub title: Option<String>,
    pub type_: Option<String>,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub permalink: Option<Option<String>>,
    pub body: Option<String>,
    pub body_html: Option<String>,
    pub creator_id: Option<i32>,
    pub board_id: Option<i32>,
    pub is_removed: Option<bool>,
    pub is_locked: Option<bool>,
    pub updated: Option<NaiveDateTime>,
    pub is_deleted: Option<bool>,
    pub is_nsfw: Option<bool>,
    pub is_stickied: Option<bool>,
    pub image: Option<String>,
}
