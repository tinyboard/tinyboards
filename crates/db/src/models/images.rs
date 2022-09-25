use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Images {
    id: i32,
    img_state: Nullable<String>,
    img_number: Nullable<i32>,
    img_text: Nullable<String>,
}