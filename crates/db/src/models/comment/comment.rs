use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Comment {
    id: i32,
    creator_id: i32,
    post_id: i32,
    parent_id: Option<i32>,
    body: String,
    removed: bool,
    read: bool,
    published: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime,
}