use crate::schema::boards;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = boards)]
pub struct Board {
    pub id: i32,
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub creator_id: i32,
    pub is_banned: bool,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub is_deleted: bool,
    pub is_nsfw: bool,
    pub is_hidden: bool,
    pub actor_id: String,
    pub local: bool,
    pub private_key: Option<String>,
    pub public_key: Option<String>,
    pub subscribers_url: String,
    pub inbox_url: String,
    pub shared_inbox_url: Option<String>,
    pub last_refreshed_date: NaiveDateTime,
    pub instance_id: i32,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, Insertable, AsChangeset)]
#[diesel(table_name = boards)]
pub struct BoardForm {
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub creator_id: Option<i32>,
    pub is_banned: Option<bool>,
    pub updated: Option<Option<NaiveDateTime>>,
    pub is_deleted: Option<bool>,
    pub is_nsfw: Option<bool>,
    pub is_hidden: Option<bool>,
    pub actor_id: Option<String>,
    pub local: Option<bool>,
    pub private_key: Option<Option<String>>,
    pub public_key: Option<Option<String>>,
    pub subscribers_url: Option<String>,
    pub inbox_url: Option<String>,
    pub shared_inbox_url: Option<Option<String>>,
    pub last_refreshed_date: Option<NaiveDateTime>,
    pub instance_id: Option<i32>,
}

/// A safe representation of board, without the sensitive info
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = boards)]
pub struct BoardSafe {
    pub id: i32,
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub creation_date: chrono::NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub is_deleted: bool,
    pub is_nsfw: bool,
    pub is_hidden: bool,
}
