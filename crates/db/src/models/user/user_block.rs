use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use crate::schema::user_block;
use chrono::NaiveDateTime;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = user_block)]
pub struct UserBlock {
    pub id: i32,
    pub user_id: i32,
    pub target_id: i32,
    pub published: NaiveDateTime,
}