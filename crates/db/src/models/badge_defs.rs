use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct BadgeDefs{
    id: i32,
    badge_name: String,
    badge_description: String,
    badge_kind: i16,
    badge_rank: i16,
    qualification_expr: Nullable<String>,
}