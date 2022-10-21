use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use crate::schema::board_user_ban;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = board_user_ban)]
pub struct BoardUserBan {
    pub id: i32,
    pub board_id: i32,
    pub user_id: i32,
    pub published: NaiveDateTime,
    pub expires: Option<NaiveDateTime>,
}