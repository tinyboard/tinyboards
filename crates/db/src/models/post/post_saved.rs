use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct PostSaved {
    id: i32,
    post_id: i32,
    user_id: i32,
    published: chrono::NaiveDateTime,
}