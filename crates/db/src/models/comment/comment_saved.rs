use crate::schema::comment_saved;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = comment_saved)]
pub struct CommentSaved {
    pub id: i32,
    pub comment_id: i32,
    pub user_id: i32,
    pub creation_date: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = comment_saved)]
pub struct CommentSavedForm {
    pub comment_id: i32,
    pub user_id: i32,
}
