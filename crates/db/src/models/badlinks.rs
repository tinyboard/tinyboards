use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Queryable)]
pub struct BadLinks {
    id: i32,
    reason: i32,
    link: String,
    autoban: Nullable<Bool>
}