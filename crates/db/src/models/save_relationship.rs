use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct SaveRelationship {
    id: i32,
    user_id: i32,
    submission_id: i32,
}