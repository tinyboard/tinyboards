use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct ModActions {
    id: i32,
    user_id: i32,
    board_id: i32,
    kind: String,
    target_user_id: i32,
    target_submission_id: i32,
    target_comment_id: i32,
    note: Option<String>,
    created_utc: i64,
}
