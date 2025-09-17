use crate::schema::post_votes;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = post_votes)]
pub struct PostVote {
    pub id: i32,
    pub post_id: i32,
    pub user_id: i32,
    pub score: i16,
}

#[derive(Clone, Insertable, AsChangeset)]
#[diesel(table_name = post_votes)]
pub struct PostVoteForm {
    pub post_id: i32,
    pub user_id: i32,
    pub score: i16,
}
