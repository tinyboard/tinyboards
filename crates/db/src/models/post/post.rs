use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Post {
    id: i32,
    name: String,
    url: Option<String>,
    body: String,
    creator_id: i32,
    board_id: i32,
    removed: bool,
    locked: bool,
    published: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime,
}