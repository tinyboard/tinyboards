use diesel::prelude::*;
use diesel_ltree::Ltree;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Comment {
    pub id: i32,
    pub creator_id: i32,
    pub post_id: i32,
    pub body: String,
    pub removed: bool,
    pub published: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
    pub deleted: bool,
    // #[serde(with = "LtreeDef")]
    pub path: Ltree,
}
