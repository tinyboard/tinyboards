use crate::schema::user_ban;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_ban)]
pub struct UserBan {
    pub id: i32,
    pub user_id: i32,
    pub creation_date: NaiveDateTime,
}
