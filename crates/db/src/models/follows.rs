use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Follows {
    id: i32,
    user_id: i32,
    target_id: i32,
    created_utc: i64,
    get_notifs: bool,
}
