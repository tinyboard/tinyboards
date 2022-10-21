use crate::schema::comment_like;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = comment_like)]
pub struct CommentLike {
    pub id: i32,
    pub user_id: i32,
    pub comment_id: i32,
    pub score: i16,
    pub published: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = comment_like)]
pub struct CommentLikeForm {
    pub comment_id: i32,
    pub user_id: i32,
    pub score: i16,
}
