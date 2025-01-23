use crate::schema::comment_votes;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = comment_votes)]
pub struct CommentVote {
    pub id: i32,
    pub person_id: i32,
    pub comment_id: i32,
    pub score: i16,
    pub creation_date: NaiveDateTime,
    pub post_id: i32,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = comment_votes)]
pub struct CommentVoteForm {
    pub comment_id: i32,
    pub person_id: i32,
    pub score: i16,
    pub post_id: i32,
}
