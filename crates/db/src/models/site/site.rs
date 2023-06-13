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
    pub actor_id: DbUrl,
    pub instance_id: i32,
    pub icon: Option<DbUrl>,
    pub banner: Option<DbUrl>,
    pub description: Option<String>,
    pub last_refreshed_date: NaiveDateTime,
    pub inbox_url: DbUrl,
    pub private_key: Option<String>,
    pub public_key: String,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = site)]
pub struct SiteForm {
    pub name: Option<String>,
    pub sidebar: Option<Option<String>>,
    pub creation_date: Option<NaiveDateTime>,
    pub updated: Option<Option<NaiveDateTime>>,
    pub actor_id: Option<DbUrl>,
    pub instance_id: Option<i32>,
    pub icon: Option<Option<DbUrl>>,
    pub banner: Option<Option<DbUrl>>,
    pub description: Option<Option<String>>,
    pub last_refreshed_date: Option<NaiveDateTime>,
    pub inbox_url: Option<DbUrl>,
    pub private_key: Option<Option<String>>,
    pub public_key: Option<String>,
}
