use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct CommentSaved {
    id: i32,
    comment_id: i32,
    user_id: i32,
    published: chrono::NaiveDateTime,
}