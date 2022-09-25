use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct UserBlocks {
    id: i32,
    user_id: i32,
    target_id: i32,
    created_utc: i32,
}