use crate::schema::comment_like;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct CommentLike {
    id: i32,
    user_id: i32,
    comment_id: i32,
    score: i16,
    published: chrono::NaiveDateTime,
}

#[derive(Clone, Insertable, AsChangeset)]
#[diesel(table_name = comment_like)]
pub struct CommentLikeForm {
    pub comment_id: i32,
    pub user_id: i32,
    pub score: i16,
}
