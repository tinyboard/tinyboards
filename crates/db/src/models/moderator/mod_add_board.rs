use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::schema::mod_add_board;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = mod_add_board)]

pub struct ModAddBoard {
    id: i32,
    mod_user_id: i32,
    other_user_id: i32,
    board_id: i32,
    removed: Option<bool>,
    when_: NaiveDateTime,
}