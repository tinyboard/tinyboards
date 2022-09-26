use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Domains {
    id: i32,
    domain: String,
    can_submit: bool,
    can_comment: bool,
    reason: i32,
    show_thumbnail: bool,
    embed_function: Option<String>,
    embed_template: Option<String>,
}
