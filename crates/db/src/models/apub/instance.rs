use crate::schema::instance;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = instance)]
pub struct Instance {
    pub id: i32,
    pub domain: String,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = instance)]
pub struct InstanceForm {
    pub domain: String,
    pub updated: Option<NaiveDateTime>,
}