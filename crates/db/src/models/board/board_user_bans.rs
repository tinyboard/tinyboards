use crate::schema::board_user_bans;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = board_user_bans)]
pub struct BoardUserBan {
    pub id: i32,
    pub board_id: i32,
    pub user_id: i32,
    pub creation_date: NaiveDateTime,
    pub expires: Option<NaiveDateTime>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = board_user_bans)]
pub struct BoardUserBanForm {
    pub board_id: Option<i32>,
    pub user_id: Option<i32>,
    pub creation_date: Option<NaiveDateTime>,
    pub expires: Option<NaiveDateTime>,
}


// Crud implementation removed - using the existing one from impls/board/board_user_bans.rs
