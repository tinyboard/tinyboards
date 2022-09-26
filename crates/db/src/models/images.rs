use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Images {
    id: i32,
    img_state: Option<String>,
    img_number: Option<i32>,
    img_text: Option<String>,
}
