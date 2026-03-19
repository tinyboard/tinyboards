use crate::schema::{
    board_aggregates, comment_aggregates, flair_aggregates, post_aggregates, reaction_aggregates,
    site_aggregates, stream_aggregates, user_aggregates,
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// All aggregate tables are read-only (maintained by database triggers).
// No insert or update forms are provided.

/// Aggregated statistics for a post.
/// Field order matches schema.rs column order exactly.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = post_aggregates)]
pub struct PostAggregates {
    pub id: Uuid,
    pub post_id: Uuid,
    pub board_id: Uuid,
    pub creator_id: Uuid,
    pub comments: i64,
    pub score: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    pub hot_rank: i32,
    pub hot_rank_active: i32,
    pub controversy_rank: f64,
    pub is_featured_board: bool,
    pub is_featured_local: bool,
    pub newest_comment_time: DateTime<Utc>,
    pub newest_comment_time_necro: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Aggregated statistics for a comment.
/// Field order matches schema.rs column order exactly.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = comment_aggregates)]
pub struct CommentAggregates {
    pub id: Uuid,
    pub comment_id: Uuid,
    pub score: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    pub child_count: i32,
    pub hot_rank: i32,
    pub controversy_rank: f64,
    pub created_at: DateTime<Utc>,
}

/// Aggregated statistics for a board.
/// Field order matches schema.rs column order exactly.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = board_aggregates)]
pub struct BoardAggregates {
    pub id: Uuid,
    pub board_id: Uuid,
    pub subscribers: i64,
    pub posts: i64,
    pub comments: i64,
    pub users_active_day: i64,
    pub users_active_week: i64,
    pub users_active_month: i64,
    pub users_active_half_year: i64,
    pub created_at: DateTime<Utc>,
}

/// Aggregated statistics for a user.
/// Field order matches schema.rs column order exactly.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_aggregates)]
pub struct UserAggregates {
    pub id: Uuid,
    pub user_id: Uuid,
    pub post_count: i64,
    pub post_score: i64,
    pub comment_count: i64,
    pub comment_score: i64,
    pub created_at: DateTime<Utc>,
}

/// Aggregated statistics for the site.
/// Field order matches schema.rs column order exactly.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = site_aggregates)]
pub struct SiteAggregates {
    pub id: Uuid,
    pub site_id: Uuid,
    pub users: i64,
    pub posts: i64,
    pub comments: i64,
    pub boards: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    pub users_active_day: i64,
    pub users_active_week: i64,
    pub users_active_month: i64,
    pub users_active_half_year: i64,
    pub created_at: DateTime<Utc>,
}

/// Aggregated statistics for a stream.
/// Field order matches schema.rs column order exactly.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = stream_aggregates)]
pub struct StreamAggregates {
    pub id: Uuid,
    pub stream_id: Uuid,
    pub flair_subscription_count: i32,
    pub board_subscription_count: i32,
    pub total_subscription_count: i32,
    pub follower_count: i32,
    pub posts_last_day: i32,
    pub posts_last_week: i32,
    pub posts_last_month: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Aggregated statistics for a flair template.
/// Field order matches schema.rs column order exactly.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = flair_aggregates)]
pub struct FlairAggregates {
    pub id: Uuid,
    pub flair_template_id: Uuid,
    pub total_usage_count: i32,
    pub post_usage_count: i32,
    pub user_usage_count: i32,
    pub active_user_count: i32,
    pub usage_last_day: i32,
    pub usage_last_week: i32,
    pub usage_last_month: i32,
    pub avg_post_score: BigDecimal,
    pub total_post_comments: i32,
    pub total_post_score: i32,
    pub trending_score: BigDecimal,
    pub hot_rank: BigDecimal,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Aggregated reaction counts per emoji per content item.
/// Field order matches schema.rs column order exactly.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = reaction_aggregates)]
pub struct ReactionAggregates {
    pub id: Uuid,
    pub post_id: Option<Uuid>,
    pub comment_id: Option<Uuid>,
    pub emoji: String,
    pub count: i32,
}
