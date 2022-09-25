use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Titles {
    id: i32,
    is_before: Bool,
    title_text: String,
    qualification_expr: String,
    requirement_string: String,
    title_color: String,
    bg_color_1: Nullable<String>,
    bg_color_2: Nullable<String>,
    gradient_angle: i32,
    box_shadow_color: Nullable<String>,
    text_shadow_color: Nullable<String>,
}