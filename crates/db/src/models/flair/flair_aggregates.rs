use crate::schema::flair_aggregates;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = flair_aggregates)]
pub struct FlairAggregates {
    pub id: i32,
    pub flair_template_id: i32,
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
    pub last_used_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
