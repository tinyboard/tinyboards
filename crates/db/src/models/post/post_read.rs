use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::schema::post_read;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = post_read)]
pub struct PostRead {
    pub id: i32,
    pub post_id: i32,
    pub person_id: i32,
    pub creation_date: chrono::NaiveDateTime,
}

#[derive(Clone, Insertable, AsChangeset)]
#[diesel(table_name = post_read)]
pub struct PostReadForm {
    pub post_id: i32,
    pub person_id: i32,
}