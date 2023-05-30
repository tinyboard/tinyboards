use crate::schema::user_blocks;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_blocks)]
pub struct UserBlock {
    pub id: i32,
    pub person_id: i32,
    pub target_id: i32,
    pub creation_date: NaiveDateTime,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = user_blocks)]
pub struct UserBlockForm {
    pub person_id: i32,
    pub target_id: i32,
}
