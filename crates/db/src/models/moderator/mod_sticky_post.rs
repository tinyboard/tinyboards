use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Deserialize, Serialize)]
pub struct ModStickyPost {
    id: i32,
    mod_user_id: i32,
    post_id: i32,
    stickied: bool,
    when_: chrono::NaiveDateTime,
}