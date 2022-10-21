use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use crate::schema::user_ban;
use chrono::NaiveDateTime;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = user_ban)]
pub struct UserBan {
    pub id: i32,
    pub user_id: i32,
    pub published: NaiveDateTime,
}