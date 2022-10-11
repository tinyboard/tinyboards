use crate::schema::post;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct Post {
    id: i32,
    title: String,
    type_: String,
    url: Option<String>,
    thumbnail_url: Option<String>,
    permalink: Option<String>,
    body: String,
    creator_id: i32,
    board_id: i32,
    removed: bool,
    locked: bool,
    published: NaiveDateTime,
    updated: Option<NaiveDateTime>,
    deleted: bool,
    nsfw: bool,
    stickied: bool,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = post)]
pub struct PostForm {
    pub title: String,
    pub type_: Option<String>,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub permalink: Option<Option<String>>,
    pub body: Option<String>,
    pub creator_id: i32,
    pub board_id: i32,
    pub removed: Option<bool>,
    pub locked: Option<bool>,
    pub published: Option<NaiveDateTime>,
    pub updated: Option<NaiveDateTime>,
    pub deleted: Option<bool>,
    pub nsfw: Option<bool>,
    pub stickied: Option<bool>,
}
