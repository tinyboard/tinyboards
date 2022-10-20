use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use crate::schema::comment_saved;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = comment_saved)]
pub struct CommentSaved {
    pub id: i32,
    pub comment_id: i32,
    pub user_id: i32,
    pub published: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = comment_saved)]
pub struct CommentSavedForm {
    pub comment_id: i32,
    pub user_id: i32,
}