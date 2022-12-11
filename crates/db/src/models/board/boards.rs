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
    pub tag_id: i32,
    pub creator_id: i32,
    pub is_banned: bool,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub is_deleted: bool,
    pub is_nsfw: bool,
    pub is_hidden: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, Insertable, AsChangeset)]
#[diesel(table_name = boards)]
pub struct BoardForm {
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub tag_id: Option<i32>,
    pub creator_id: Option<i32>,
    pub is_banned: Option<bool>,
    pub updated: Option<Option<NaiveDateTime>>,
    pub is_deleted: Option<bool>,
    //pub is_nsfw: Option<bool>,
    pub is_hidden: Option<bool>,
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
