use crate::schema::{
    board_aggregates, comment_aggregates, post_aggregates, site_aggregates, stream_aggregates,
};
use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(
    PartialEq, Debug, Serialize, Deserialize, Clone, Queryable, Associations, Identifiable,
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
    pub child_count: i32,
    pub hot_rank: i32,
    pub controversy_rank: f64,
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
    PartialEq, Debug, Serialize, Deserialize, Clone, Queryable, Associations, Identifiable,
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
    pub creation_date: NaiveDateTime,
    pub newest_comment_time_necro: Option<NaiveDateTime>,
    pub newest_comment_time: NaiveDateTime,
    pub featured_board: bool,
    pub featured_local: bool,
    pub hot_rank: i32,
    pub hot_rank_active: i32,
    pub board_id: i32,
    pub creator_id: i32,
    pub controversy_rank: f64,
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
    pub upvotes: i64,
    pub downvotes: i64,
}

#[derive(
    PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Queryable, Associations, Identifiable,
)]
#[diesel(table_name = stream_aggregates)]
#[diesel(belongs_to(crate::models::stream::stream::Stream))]
pub struct StreamAggregates {
    pub id: i32,
    pub stream_id: i32,
    pub flair_subscription_count: i32,
    pub board_subscription_count: i32,
    pub total_subscription_count: i32,
    pub follower_count: i32,
    pub posts_last_day: i32,
    pub posts_last_week: i32,
    pub posts_last_month: i32,
    pub creation_date: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl Default for StreamAggregates {
    fn default() -> Self {
        Self {
            id: 0,
            stream_id: 0,
            flair_subscription_count: 0,
            board_subscription_count: 0,
            total_subscription_count: 0,
            follower_count: 0,
            posts_last_day: 0,
            posts_last_week: 0,
            posts_last_month: 0,
            creation_date: chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
            updated_at: None,
        }
    }
}

