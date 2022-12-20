use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    id: i32,
    user_id: i32,
    token_encrypted: String,
    creation_date: chrono::NaiveDateTime,
}
