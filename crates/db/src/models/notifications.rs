use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Notifications {
    id: i32,
    user_id: i32,
    comment_id: Option<i32>,
    submission_id: Option<i32>,
    notification_read: bool,
}
