use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Contributors {
    id: i32,
    user_id: i32,
    board_id: i32,
    created_utc: i64,
    is_active: bool,
    approving_mod_id: i32,
}
