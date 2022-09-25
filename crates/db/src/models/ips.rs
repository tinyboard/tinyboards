use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Ips {
    id: i32,
    addr: String,
    reason: String,
    banned_by: i32
}