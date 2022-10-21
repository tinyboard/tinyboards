use serde::{Deserialize, Serialize};
use diesel::{Queryable, Identifiable, Associations};
use crate::schema::{
    post_aggregates,
    user_aggregates,
    comment_aggregates,
    board_aggregates,
};
use chrono::NaiveDateTime;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[derive(Queryable, Associations, Identifiable)]
#[diesel(table_name = comment_aggregates)]
#[diesel(belongs_to(crate::models::comment::comment::Comment))]
pub struct CommentAggregates {
    pub id: i32,
    pub comment_id: i32,
    pub score: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    pub published: NaiveDateTime,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[derive(Queryable, Associations, Identifiable)]
#[diesel(table_name = board_aggregates)]
#[diesel(belongs_to(crate::models::board::board::Board))]
pub struct BoardAggregates {
    pub id: i32,
    pub board_id: i32,
    pub subscribers: i64,
    pub posts: i64,
    pub comments: i64,
    pub published: NaiveDateTime,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[derive(Queryable, Associations, Identifiable)]
#[diesel(table_name = post_aggregates)]
#[diesel(belongs_to(crate::models::post::post::Post))]
pub struct PostAggregates {
    pub id: i32,
    pub post_id: i32,
    pub comments: i64,
    pub score: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    pub stickied: bool,
    pub published: NaiveDateTime,
    pub newest_comment_time: NaiveDateTime,
}


#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[derive(Queryable, Associations, Identifiable)]
#[diesel(table_name = user_aggregates)]
#[diesel(belongs_to(crate::models::user::user::User))]
pub struct UserAggregates {
    pub id: i32,
    pub user_id: i32,
    pub post_count: i64,
    pub post_score: i64,
    pub comment_count: i64,
    pub comment_score: i64,
}