use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct CommentLike {
    id: i32,
    user_id: i32,
    comment_id: i32,
    post_id: i32,
    score: i16,
    published: chrono::NaiveDateTime,
}