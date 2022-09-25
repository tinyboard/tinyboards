use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct BadPics{
    id: i32,
    badpic_description: Nullable<String>,
    phash: Nullable<String>,
    ban_reason: Nullable<String>,
    ban_time: Nullable<i32>
}