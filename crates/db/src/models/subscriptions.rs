use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Subscriptions {
    id: i32,
    user_id: i32,
    board_id: i32,
    created_utc: i64,
    is_active: bool,
    get_notifs: bool,
}
