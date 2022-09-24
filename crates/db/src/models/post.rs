use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
}
