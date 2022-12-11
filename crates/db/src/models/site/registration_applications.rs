use crate::schema::registration_applications;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = registration_applications)]
pub struct RegistrationApplication {
    pub id: i32,
    pub user_id: i32,
    pub answer: String,
    pub admin_id: Option<i32>,
    pub deny_reason: Option<String>,
    pub creation_date: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = registration_applications)]
pub struct RegistrationApplicationForm {
    pub user_id: i32,
    pub answer: Option<String>,
    pub admin_id: Option<Option<i32>>,
    pub deny_reason: Option<Option<String>>,
}
