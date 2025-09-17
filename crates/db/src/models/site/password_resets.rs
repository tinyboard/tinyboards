use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use crate::schema::password_resets;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = password_resets)]
pub struct PasswordReset {
    pub id: i32,
    pub reset_token: String,
    pub creation_date: NaiveDateTime,
    pub user_id: i32,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = password_resets)]
pub struct PasswordResetForm {
    pub reset_token: String,
    pub user_id: i32,
}