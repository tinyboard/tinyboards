use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct PostRels {
    id: i32,
    post_id: i32,
    board_id: i32,
}