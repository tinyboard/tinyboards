use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Deserialize, Serialize)]
pub struct ModAddBoard {
    id: i32,
    mod_user_id: i32,
    other_user_id: i32,
    board_id: i32,
    removed: Option<bool>,
    when_: chrono::NaiveDateTime,
}