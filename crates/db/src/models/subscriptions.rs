use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Subscriptions {
    id: i32,
    user_id: i32,
    board_id: i32,
    created_utc: i32,
    is_active: Bool,
    get_notifs: Bool,
}