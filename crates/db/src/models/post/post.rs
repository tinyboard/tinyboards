use crate::schema::post;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = post)]
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
    pub removed: bool,
    pub locked: bool,
    pub published: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub deleted: bool,
    pub nsfw: bool,
    pub stickied: bool,
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
    pub body_html: Option<String>,
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
