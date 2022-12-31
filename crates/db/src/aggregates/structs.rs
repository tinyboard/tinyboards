use crate::schema::{board_aggregates, comment_aggregates, post_aggregates, user_aggregates};
use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(
    PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Queryable, Associations, Identifiable,
)]
#[diesel(table_name = comment_aggregates)]
#[diesel(belongs_to(crate::models::comment::comments::Comment))]
pub struct CommentAggregates {
    pub id: i32,
    pub comment_id: i32,
    pub score: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    pub creation_date: NaiveDateTime,
}

#[derive(
    PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Queryable, Associations, Identifiable,
)]
#[diesel(table_name = board_aggregates)]
#[diesel(belongs_to(crate::models::board::boards::Board))]
pub struct BoardAggregates {
    pub id: i32,
    pub board_id: i32,
    pub subscribers: i64,
    pub posts: i64,
    pub comments: i64,
    pub creation_date: NaiveDateTime,
}

#[derive(
    PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Queryable, Associations, Identifiable,
)]
#[diesel(table_name = post_aggregates)]
#[diesel(belongs_to(crate::models::post::posts::Post))]
pub struct PostAggregates {
    pub id: i32,
    pub post_id: i32,
    pub comments: i64,
    pub score: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    pub stickied: bool,
    pub creation_date: NaiveDateTime,
    pub newest_comment_time: NaiveDateTime,
}

#[derive(
    PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Queryable, Associations, Identifiable,
)]
#[diesel(table_name = user_aggregates)]
#[diesel(belongs_to(crate::models::user::users::User))]
pub struct UserAggregates {
    pub id: i32,
    pub user_id: i32,
    pub post_count: i64,
    pub post_score: i64,
    pub comment_count: i64,
    pub comment_score: i64,
    pub rep: i64,
}
