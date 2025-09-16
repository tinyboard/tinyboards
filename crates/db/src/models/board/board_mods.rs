use crate::schema::board_mods;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

pub enum ModPerms {
    None,
    Config,
    Appearance,
    Content,
    Users,
    Full,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = board_mods)]
pub struct BoardModerator {
    pub id: i32,
    pub board_id: i32,
    pub user_id: i32,
    pub creation_date: NaiveDateTime,
    pub permissions: i32,
    pub rank: i32,
    pub invite_accepted: bool,
    pub invite_accepted_date: Option<NaiveDateTime>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, Insertable, AsChangeset)]
#[diesel(table_name = board_mods)]
pub struct BoardModeratorForm {
    pub board_id: Option<i32>,
    pub user_id: Option<i32>,
    pub permissions: Option<i32>,
    pub rank: Option<i32>,
    pub invite_accepted: Option<bool>,
    pub invite_accepted_date: Option<Option<NaiveDateTime>>,
}
