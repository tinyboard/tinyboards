use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Categories {
    id: i32,
    category_name: String,
    category_description: String,
    category_icon: String,
    category_color: Option<String>,
    visible: bool,
    is_nsfw: bool,
}
