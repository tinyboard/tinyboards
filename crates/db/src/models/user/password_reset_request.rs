use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    id: i32,
    user_id: i32,
    token_encrypted: String,
    published: chrono::NaiveDateTime,
}