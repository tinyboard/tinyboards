use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Categories {
    id: i32,
    category_name: String,
    category_description: String,
    category_icon: String,
    category_color: Nullable<String>,
    visible: Bool,
    is_nsfw: Bool
}