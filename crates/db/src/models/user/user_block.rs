use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct UserBlock {
    pub id: i32,
    pub user_id: i32,
    pub target_id: i32,
    pub published: chrono::NaiveDateTime,
}