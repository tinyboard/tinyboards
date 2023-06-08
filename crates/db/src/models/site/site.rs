use crate::schema::site;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = site)]
pub struct Site {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub creator_id: i32,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = site)]
pub struct SiteForm {
    pub name: Option<String>,
    pub description: Option<String>,
    pub creator_id: Option<i32>,
    pub updated: Option<Option<NaiveDateTime>>,
}
