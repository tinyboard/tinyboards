use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct UserAgents {
    id: i32,
    kwd: String,
    reason: String,
    banned_by: i32,
    mock: String,
    status_code: i32,
}