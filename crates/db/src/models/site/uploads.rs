use crate::{schema::uploads, newtypes::DbUrl};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = uploads)]
pub struct Upload {
    pub id: i32,
    pub person_id: i32,
    pub original_name: String,
    pub file_name: String,
    pub file_path: String,
    pub upload_url: DbUrl,
    pub creation_date: NaiveDateTime,
    pub size: i64,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = uploads)]
pub struct UploadForm {
    pub person_id: i32,
    pub original_name: String,
    pub file_name: String,
    pub file_path: String,
    pub upload_url: Option<DbUrl>,
    pub size: i64,
}
