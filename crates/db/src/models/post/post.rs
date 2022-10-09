use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use crate::schema::post;

#[derive(Queryable, Serialize, Deserialize)]
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
    published: chrono::NaiveDateTime,
    updated: Option<chrono::NaiveDateTime>,
    nsfw: bool,
    stickied: bool,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = post)]
pub struct PostForm {
    pub title: Option<String>,
    pub type_: Option<String>,
    pub url: Option<Option<String>>,
    pub thumbnail_url: Option<Option<String>>,
    pub permalink: Option<Option<String>>,
    pub body: Option<String>,
    pub creator_id: Option<i32>,
    pub board_id: Option<i32>,
    pub removed: Option<bool>,
    pub locked: Option<bool>,
    pub published: Option<chrono::NaiveDateTime>,
    pub updated: Option<chrono::NaiveDateTime>,
    pub nsfw: Option<bool>,
    pub stickied: Option<bool>,
}