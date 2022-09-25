use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct BadgeDefs{
    id: i32,
    badge_name: Nullable<String>,
    badge_description: Nullable<String>,
    badge_kind: Nullable<i16>,
    badge_rank: Nullable<i16>,
    qualification_expr: Nullable<String>
}