use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Domains {
    id: i32,
    domain: String,
    can_submit: Bool,
    can_comment: Bool,
    reason: i32,
    show_thumbnail: Bool,
    embed_function: Nullable<String>,
    embed_template: Nullable<String>
}