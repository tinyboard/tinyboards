use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct BoardSite {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub creator_id: i32,
    pub creation_date: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
    pub enable_downvotes: bool,
    pub open_registration: bool,
    pub enable_nsfw: bool,
}
