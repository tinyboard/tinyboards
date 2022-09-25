use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct BadPics{
    id: i32,
    badpic_description: Nullable<String>,
    phash: String,
    ban_reason: String,
    ban_time: i32,
}