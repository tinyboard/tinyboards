use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Deserialize, Serialize)]
pub struct ModRemoveComment {
    id: i32,
    mod_user_id: i32,
    comment_id: i32,
    reason: Option<String>,
    removed: Option<bool>,
    when_: chrono::NaiveDateTime,
}