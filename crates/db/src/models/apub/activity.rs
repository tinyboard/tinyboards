use crate::{schema::activity, newtypes::DbUrl};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = activity)]
pub struct Activity {
    pub id: i32,
    pub ap_id: String,
    pub data: Value,
    pub local: bool,
    pub sensitive: bool,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = activity)]
pub struct ActivityForm {
    pub ap_id: Option<String>,
    pub data: Option<Value>,
    pub local: Option<bool>,
    pub sensitive: Option<bool>,
    pub updated: Option<Option<NaiveDateTime>>,
}