use crate::schema::board_person_bans;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = board_person_bans)]
pub struct BoardPersonBan {
    pub id: i32,
    pub board_id: i32,
    pub person_id: i32,
    pub creation_date: NaiveDateTime,
    pub expires: Option<NaiveDateTime>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = board_person_bans)]
pub struct BoardPersonBanForm {
    pub board_id: i32,
    pub person_id: i32,
    pub expires: Option<NaiveDateTime>,
}
