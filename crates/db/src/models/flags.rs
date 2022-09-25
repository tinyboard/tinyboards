use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Flags {
    id: i32,
    post_id: i32,
    user_id: i32,
    created_utc: i32,
}