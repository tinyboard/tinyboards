use crate::schema::user_board_blocks;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_board_blocks)]
pub struct BoardBlock {
    pub id: i32,
    pub person_id: i32,
    pub board_id: i32,
    pub creation_date: NaiveDateTime,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = user_board_blocks)]
pub struct BoardBlockForm {
    pub person_id: i32,
    pub board_id: i32,
}
