use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct BoardSite {
    id: i32,
    name: String,
    description: Option<String>,
    creator_id: i32,
    published: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime,
}