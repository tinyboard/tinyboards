use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct PostRead {
    pub id: i32,
    pub post_id: i32,
    pub user_id: i32,
    pub creation_date: chrono::NaiveDateTime,
}
