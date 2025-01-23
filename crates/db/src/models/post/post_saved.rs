use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use crate::schema::post_saved;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = post_saved)]
pub struct PostSaved {
    pub id: i32,
    pub post_id: i32,
    pub person_id: i32,
    pub creation_date: chrono::NaiveDateTime,
}

#[derive(Clone, Insertable, AsChangeset)]
#[diesel(table_name = post_saved)]
pub struct PostSavedForm {
    pub post_id: i32,
    pub person_id: i32,
}