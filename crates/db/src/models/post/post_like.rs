use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct PostLike {
    pub id: i32,
    pub post_id: i32,
    pub user_id: i32,
    pub score: i16,
}