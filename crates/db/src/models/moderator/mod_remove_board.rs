use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Deserialize, Serialize)]
pub struct ModRemoveBoard {
    id: i32,
    mod_user_id: i32,
    board_id: i32,
    reason: Option<String>,
    removed: Option<bool>,
    expires: Option<chrono::NaiveDateTime>,
    when_: chrono::NaiveDateTime,
}