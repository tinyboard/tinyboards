use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct BadWords {
    id: i32,
    keyword: Nullable<String>,
    regex: Nullable<String>
}