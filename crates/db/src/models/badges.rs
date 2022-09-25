use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Badges{
    id: i32,
    user_id: i32,
    badge_id: i32,
    badge_description: Nullable<String>,
    badge_url: Nullable<String>,
    created_utc: i32
}