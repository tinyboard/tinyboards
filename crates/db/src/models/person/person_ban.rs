use crate::schema::person_ban;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = person_ban)]
pub struct PersonBan {
    pub id: i32,
    pub person_id: i32,
    pub creation_date: NaiveDateTime,
}
