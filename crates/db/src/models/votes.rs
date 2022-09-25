use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Votes {
    id: i64,
    user_id: i32,
    vote_type: i32,
    submission_id: i32,
    created_utc: i32,
    creation_ip: String,
    app_id: Nullable<i32>,
}