use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Titles {
    id: i32,
    is_before: bool,
    title_text: String,
    qualification_expr: String,
    requirement_string: String,
    title_color: String,
    bg_color_1: Option<String>,
    bg_color_2: Option<String>,
    gradient_angle: i32,
    box_shadow_color: Option<String>,
    text_shadow_color: Option<String>,
}
