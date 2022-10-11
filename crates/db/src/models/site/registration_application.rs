use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct RegistrationApplication{
    id: i32,
    user_id: i32,
    answer: String,
    admin_id: Option<i32>,
    deny_reason: Option<String>,
    published: chrono::NaiveDateTime,
}