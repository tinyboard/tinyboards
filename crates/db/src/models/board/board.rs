use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Board {
    id: i32,
    name: String,
    title: String,
    description: Option<String>,
    tag_id: i32,
    creator_id: i32,
    removed: bool,
    published: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime,
    deleted: bool,
    nsfw: bool,
}

/// A safe representation of board, without the sensitive info
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", table_name = "board")]
pub struct BoardSafe {
  pub id: i32,
  pub name: String,
  pub title: String,
  pub description: Option<String>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub nsfw: bool,
}