use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use crate::schema::password_resets;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = password_resets)]
pub struct PasswordReset {
    pub id: i32,
    pub person_id: i32,
    pub reset_token: String,
    pub creation_date: NaiveDateTime,
}

// TODO - rename local_person_id to local_user_id in migration
#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = password_resets)]
pub struct PasswordResetForm {
    pub local_person_id: i32,
    pub reset_token: String,
}