use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use crate::schema::board_moderator;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = board_moderator)]
pub struct BoardModerator {
    pub id: i32,
    pub board_id: i32,
    pub user_id: i32,
    pub published: NaiveDateTime,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[derive(Insertable, AsChangeset)]
#[diesel(table_name = board_moderator)]
pub struct BoardModeratorForm {
    pub board_id: i32,
    pub user_id: i32,
}