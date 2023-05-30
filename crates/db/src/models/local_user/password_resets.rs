use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::schema::password_resets;
use chrono::NaiveDateTime;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = password_resets)]
pub struct PasswordReset {
    id: i32,
    reset_token: String,
    creation_date: NaiveDateTime,
    local_person_id: i32,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, Insertable, AsChangeset)]
#[diesel(table_name = password_resets)]
pub struct PasswordResetForm {
    pub reset_token: Option<String>,
    pub local_person_id: Option<i32>,
}