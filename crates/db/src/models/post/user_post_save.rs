use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use crate::schema::user_post_save;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = user_post_save)]
pub struct PostSaved {
    pub id: i32,
    pub post_id: i32,
    pub user_id: i32,
    pub creation_date: chrono::NaiveDateTime,
}

#[derive(Clone, Insertable, AsChangeset)]
#[diesel(table_name = user_post_save)]
pub struct PostSavedForm {
    pub post_id: i32,
    pub user_id: i32,
}