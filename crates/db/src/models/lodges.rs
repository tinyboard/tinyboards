use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Lodges {
    id: i32,
    lodge_name: String,
    lodge_color: String,
    lodge_description: String,
    user_id: i32,
    board_id: i32,
    created_utc: i64,
}