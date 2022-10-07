use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Post {
    id: i32,
    name: String,
    type_: Option<String>,
    url: Option<String>,
    thumbnail_url: Option<String>,
    permalink: Option<String>,
    body: String,
    creator_id: i32,
    board_id: i32,
    removed: bool,
    locked: bool,
    published: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime,
    nsfw: bool,
    stickied: bool,
}