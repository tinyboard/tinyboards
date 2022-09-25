use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct ModActions {
    id: i32,
    user_id: i32,
    board_id: i32,
    kind: String,
    target_user_id: i32,
    target_submission_id: i32,
    target_comment_id: i32,
    note: Nullable<String>,
    created_utc: i32
}