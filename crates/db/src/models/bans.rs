use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Bans {
    id: i32,
    user_id: i32,
    board_id: i32,
    created_utc: i32,
    banning_mod_id: i32,
    is_active: i32,
    mod_note: String,
}