use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Deserialize, Serialize)]
pub struct ModBan {
    id: i32,
    mod_user_id: i32,
    other_user_id: i32,
    reason: Option<String>,
    banned: Option<bool>,
    expires: Option<chrono::NaiveDateTime>,
    when: chrono::NaiveDateTime,
}