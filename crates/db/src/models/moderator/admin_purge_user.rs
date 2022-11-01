use crate::schema::admin_purge_user;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::NaiveDateTime;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = admin_purge_user)]
pub struct AdminPurgeUser {
    pub id: i32,
    pub admin_id: i32,
    pub user_id: i32,
    pub reason: Option<String>,
    pub when_: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = admin_purge_user)]
pub struct AdminPurgeCommentForm {
    pub admin_id: i32,
    pub user_id: i32,
    pub reason: Option<Option<String>>,
}