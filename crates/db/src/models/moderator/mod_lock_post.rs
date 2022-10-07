use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Deserialize, Serialize)]
pub struct ModLockPost {
    id: i32,
    mod_user_id: i32,
    post_id: i32,
    locked: Option<bool>,
    when_: chrono::NaiveDateTime,
}