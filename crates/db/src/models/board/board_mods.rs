use crate::schema::board_mods;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = board_mods)]
pub struct BoardModerator {
    pub id: i32,
    pub board_id: i32,
    pub user_id: i32,
    pub creation_date: NaiveDateTime,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, Insertable, AsChangeset)]
#[diesel(table_name = board_mods)]
pub struct BoardModeratorForm {
    pub board_id: i32,
    pub user_id: i32,
}
