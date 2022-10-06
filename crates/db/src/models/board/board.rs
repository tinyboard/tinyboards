use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Board {
    id: i32,
    name: String,
    title: String,
    description: Option<String>,
    tag_id: i32,
    creator_id: i32,
    removed: bool,
    published: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime,
}