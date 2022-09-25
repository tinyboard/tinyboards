use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Notifications {
    id: i32,
    user_id: i32,
    comment_id: Nullable<i32>,
    submission_id: Nullable<i32>,
    notification_read: Bool,
}