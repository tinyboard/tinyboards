use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use crate::schema::board;
use chrono::NaiveDateTime;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = board)]
pub struct Board {
    pub id: i32,
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub tag_id: i32,
    pub creator_id: i32,
    pub removed: bool,
    pub published: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub deleted: bool,
    pub nsfw: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Insertable, AsChangeset)]
#[diesel(table_name = board)]
pub struct BoardForm {
  pub name: Option<String>,
  pub title: Option<String>,
  pub description: Option<Option<String>>,
  pub tag_id: Option<i32>,
  pub creator_id: Option<i32>,
  pub removed: Option<bool>,
  pub updated: Option<Option<NaiveDateTime>>,
  pub deleted: Option<bool>,
  pub nsfw: Option<bool>,
}


/// A safe representation of board, without the sensitive info
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = board)]
pub struct BoardSafe {
  pub id: i32,
  pub name: String,
  pub title: String,
  pub description: Option<String>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<NaiveDateTime>,
  pub deleted: bool,
  pub nsfw: bool,
}