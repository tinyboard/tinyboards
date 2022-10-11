use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct BoardSubscriber {
    pub id: i32,
    pub board_id: i32,
    pub user_id: i32,
    pub published: chrono::NaiveDateTime,
    pub pending: Option<bool>,
}