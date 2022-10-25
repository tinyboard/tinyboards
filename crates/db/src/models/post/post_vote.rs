use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use crate::schema::post_vote;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = post_vote)]
pub struct PostVote {
    pub id: i32,
    pub post_id: i32,
    pub user_id: i32,
    pub score: i16,
}

#[derive(Clone, Insertable, AsChangeset)]
#[diesel(table_name = post_vote)]
pub struct PostVoteForm {
    pub post_id: i32,
    pub user_id: i32,
    pub score: i16,
}