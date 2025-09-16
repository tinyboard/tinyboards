use crate::schema::{
    board_aggregates, comment_aggregates, post_aggregates, site_aggregates,
};
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
    pub reply_count: Option<i32>,
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
    pub users_active_day: i64,
    pub users_active_week: i64,
    pub users_active_month: i64,
    pub users_active_half_year: i64,
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

pub use crate::aggregates::user_aggregates::UserAggregates;

#[derive(
    PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Queryable, Associations, Identifiable,
)]
#[diesel(table_name = site_aggregates)]
#[diesel(belongs_to(crate::models::site::site::Site))]
pub struct SiteAggregates {
    pub id: i32,
    pub site_id: i32,
    pub users: i64,
    pub posts: i64,
    pub comments: i64,
    pub boards: i64,
    pub users_active_day: i64,
    pub users_active_week: i64,
    pub users_active_month: i64,
    pub users_active_half_year: i64,
}

