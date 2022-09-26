use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct BadPics {
    id: i32,
    badpic_description: Option<String>,
    phash: String,
    ban_reason: String,
    ban_time: i32,
}
