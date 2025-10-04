use crate::newtypes::DbUrl;
use crate::schema::content_uploads;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Queryable, Identifiable, Selectable, Associations)]
#[diesel(belongs_to(crate::models::site::uploads::Upload))]
#[diesel(belongs_to(crate::models::post::posts::Post))]
#[diesel(belongs_to(crate::models::comment::comments::Comment))]
#[diesel(table_name = content_uploads)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ContentUpload {
    pub id: i32,
    pub upload_id: i32,
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub position: Option<i32>,
}

#[derive(Clone, Default, Debug, Insertable, AsChangeset)]
#[diesel(table_name = content_uploads)]
pub struct ContentUploadForm {
    pub upload_id: i32,
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub position: Option<i32>,
}

/// Represents a content upload with full upload details
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentUploadView {
    pub id: i32,
    pub upload_id: i32,
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub position: Option<i32>,
    // Upload details
    pub file_name: String,
    pub original_name: String,
    pub upload_url: DbUrl,
    pub size: i64,
}
