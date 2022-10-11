use serde::{Deserialize, Serialize};
use diesel::Queryable;

#[derive(Queryable, Clone, Default, Debug, Serialize, Deserialize)]
pub struct CommentAggregates {
    pub id: i32,
    pub comment_id: i32,
    pub score: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    pub published: chrono::NaiveDateTime,
    pub child_count: i32,
}

#[derive(Queryable, Clone, Default, Debug, Serialize, Deserialize)]
pub struct BoardAggregates {
    pub id: i32,
    pub board_id: i32,
    pub subscribers: i64,
    pub posts: i64,
    pub comments: i64,
    pub published: chrono::NaiveDateTime,
}

#[derive(Queryable, Clone, Default, Debug, Serialize, Deserialize)]
pub struct PostAggregates {
    pub id: i32,
    pub post_id: i32,
    pub comments: i64,
    pub score: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    pub stickied: bool,
    pub published: chrono::NaiveDateTime,
    pub newest_comment_time: chrono::NaiveDateTime,
}

#[derive(Queryable, Clone, Default, Debug, Serialize, Deserialize)]
pub struct UserAggregates {
    pub id: i32,
    pub user_id: i32,
    pub post_count: i64,
    pub post_score: i64,
    pub comment_count: i64,
    pub comment_score: i64,
}
