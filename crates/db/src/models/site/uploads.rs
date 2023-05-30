use crate::schema::uploads;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = uploads)]
pub struct Upload {
    pub id: i32,
    pub user_id: i32,
    pub original_name: String,
    pub file_name: String,
    pub file_path: String,
    pub upload_url: String,
    pub creation_date: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = uploads)]
pub struct UploadForm {
    pub user_id: i32,
    pub original_name: String,
    pub file_name: String,
    pub file_path: String,
    pub upload_url: String,
}
