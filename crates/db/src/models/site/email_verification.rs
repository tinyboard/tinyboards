use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use crate::schema::email_verification;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = email_verification)]
pub struct EmailVerification {
    pub id: i32,
    pub person_id: i32,
    pub email: String,
    pub verification_code: String,
    pub published: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = email_verification)]
pub struct EmailVerificationForm {
    pub person_id: i32,
    pub email: String,
    pub verification_code: String,
}