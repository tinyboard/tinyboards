use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct CommentVotes {
    id: i32,
    user_id: i32,
    vote_type: i32,
    comment_id: i32,
    created_utc: i64,
    creation_ip: String,
    app_id: Option<i32>,
}
