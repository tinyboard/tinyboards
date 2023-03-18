use crate::schema::uploads;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};


#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = uploads)]
pub struct Upload {
    pub id: i32,
    pub filename: String,
    pub filepath: String,
    pub uploaded_at: NaiveDateTime
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, Insertable, AsChangeset)]
#[diesel(table_name = uploads)]
pub struct UploadForm {
    pub filename: String,
    pub filepath: String,
}