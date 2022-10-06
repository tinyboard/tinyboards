use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct UserMention {
    id: i32,
    recipient_id: i32,
    comment_id: i32,
    read: bool,
    published: chrono::NaiveDateTime,
}