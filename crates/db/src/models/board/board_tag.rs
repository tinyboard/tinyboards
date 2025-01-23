use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct BoardTag {
    pub id: i32,
    pub name: String,
}