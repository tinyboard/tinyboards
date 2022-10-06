use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct PrivateMessage {
    id: i32,
    creator_id: i32,
    recipient_id: i32,
    body: String,
    deleted: bool,
    read: bool,
    published: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime,
}