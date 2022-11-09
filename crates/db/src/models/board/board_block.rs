use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use crate::schema::board_block;
use chrono::NaiveDateTime;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = board_block)]
pub struct BoardBlock {
    pub id: i32,
    pub user_id: i32,
    pub board_id: i32,
    pub published: NaiveDateTime,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Insertable, AsChangeset)]
#[diesel(table_name = board_block)]
pub struct BoardBlockForm {
    pub user_id: i32,
    pub board_id: i32,
}