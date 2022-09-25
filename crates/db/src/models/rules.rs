use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Rules {
    id: i32,
    board_id: i32,
    rule_body: String,
    rule_html: String,
    created_utc: i32,
    edited_utc: i32,
}