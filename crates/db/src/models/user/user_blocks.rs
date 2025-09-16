use crate::schema::user_blocks;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_blocks)]
pub struct UserBlock {
    pub id: i32,
    pub user_id: i32,
    pub target_id: i32,
    pub creation_date: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = user_blocks)]
pub struct UserBlockForm {
    pub user_id: Option<i32>,
    pub target_id: Option<i32>,
    pub creation_date: Option<NaiveDateTime>,
}