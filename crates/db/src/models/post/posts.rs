use crate::{newtypes::DbUrl, schema::posts};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub type_: String,
    pub url: Option<DbUrl>,
    pub thumbnail_url: Option<DbUrl>,
    pub permalink: Option<DbUrl>,
    pub body: String,
    pub body_html: String,
    pub creator_id: i32,
    pub board_id: i32,
    pub is_removed: bool,
    pub is_locked: bool,
    pub creation_date: NaiveDateTime,
    pub is_deleted: bool,
    pub is_nsfw: bool,
    pub updated: Option<NaiveDateTime>,
    pub image: Option<DbUrl>,
    pub language_id: i32,
    pub ap_id: Option<DbUrl>,
    pub local: bool,
    pub featured_board: bool,
    pub featured_local: bool,
    pub title_chunk: String,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = posts)]
pub struct PostForm {
    pub title: Option<String>,
    pub type_: Option<String>,
    pub url: Option<DbUrl>,
    pub thumbnail_url: Option<DbUrl>,
    pub permalink: Option<Option<DbUrl>>,
    pub body: Option<String>,
    pub body_html: Option<String>,
    pub creator_id: Option<i32>,
    pub board_id: Option<i32>,
    pub is_removed: Option<bool>,
    pub is_locked: Option<bool>,
    pub updated: Option<NaiveDateTime>,
    pub is_deleted: Option<bool>,
    pub is_nsfw: Option<bool>,
    pub image: Option<DbUrl>,
    pub language_id: Option<i32>,
    pub creation_date: Option<NaiveDateTime>,
    pub featured_board: Option<bool>,
    pub featured_local: Option<bool>,
    pub title_chunk: Option<String>,
}
