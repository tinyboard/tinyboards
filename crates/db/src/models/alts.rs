use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Alts{
    id: i32,
    user1: i32,
    user2: i32,
    is_manual: Nullable<Bool>
}