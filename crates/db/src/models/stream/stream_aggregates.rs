use crate::schema::stream_aggregates;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Aggregated statistics for streams
/// Pre-computed counts for efficient querying
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable, Associations, Selectable)]
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

/// Form for updating stream aggregates
/// Typically used by database triggers or background jobs
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = stream_aggregates)]
pub struct UpdateStreamAggregatesForm {
    pub flair_subscription_count: Option<i32>,
    pub board_subscription_count: Option<i32>,
    pub total_subscription_count: Option<i32>,
    pub follower_count: Option<i32>,
    pub posts_last_day: Option<i32>,
    pub posts_last_week: Option<i32>,
    pub posts_last_month: Option<i32>,
    pub updated_at: Option<Option<NaiveDateTime>>,
}

/// Statistics summary for a stream
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct StreamStats {
    pub stream_id: i32,
    pub followers: i32,
    pub boards: i32,
    pub flairs: i32,
    pub total_sources: i32,
}

impl From<StreamAggregates> for StreamStats {
    fn from(agg: StreamAggregates) -> Self {
        Self {
            stream_id: agg.stream_id,
            followers: agg.follower_count,
            boards: agg.board_subscription_count,
            flairs: agg.flair_subscription_count,
            total_sources: agg.total_subscription_count,
        }
    }
}
