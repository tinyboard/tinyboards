use crate::{schema::site, newtypes::DbUrl};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = site)]
pub struct Site {
    pub id: i32,
    pub name: String,
    pub sidebar: Option<String>,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub instance_id: i32,
    pub icon: Option<DbUrl>,
    pub banner: Option<DbUrl>,
    pub description: Option<String>,
    pub last_refreshed_date: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = site)]
pub struct SiteForm {
    pub name: Option<String>,
    pub sidebar: Option<Option<String>>,
    pub creation_date: Option<NaiveDateTime>,
    pub updated: Option<Option<NaiveDateTime>>,
    pub instance_id: Option<i32>,
    pub icon: Option<Option<DbUrl>>,
    pub banner: Option<Option<DbUrl>>,
    pub description: Option<Option<String>>,
    pub last_refreshed_date: Option<NaiveDateTime>,
}
