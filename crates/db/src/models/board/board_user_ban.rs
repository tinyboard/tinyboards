use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct BoardUserBan {
    id: i32,
    board_id: i32,
    user_id: i32,
    published: chrono::NaiveDateTime,
}