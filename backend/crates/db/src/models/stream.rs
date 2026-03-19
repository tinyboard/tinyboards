use crate::enums::DbSortType;
use crate::schema::{
    stream_board_subscriptions, stream_excluded_boards, stream_excluded_users,
    stream_flair_subscriptions, stream_followers, stream_tags, stream_view_history, streams,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// streams
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = streams)]
pub struct Stream {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_public: bool,
    pub is_discoverable: bool,
    pub share_token: Option<String>,
    pub sort_type: DbSortType,
    pub time_range: Option<String>,
    pub show_nsfw: bool,
    pub max_posts_per_board: Option<i32>,
    pub last_viewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = streams)]
pub struct StreamInsertForm {
    pub creator_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_public: bool,
    pub is_discoverable: bool,
    pub share_token: Option<String>,
    pub sort_type: DbSortType,
    pub time_range: Option<String>,
    pub show_nsfw: bool,
    pub max_posts_per_board: Option<i32>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = streams)]
pub struct StreamUpdateForm {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<Option<String>>,
    pub icon: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub is_public: Option<bool>,
    pub is_discoverable: Option<bool>,
    pub share_token: Option<Option<String>>,
    pub sort_type: Option<DbSortType>,
    pub time_range: Option<Option<String>>,
    pub show_nsfw: Option<bool>,
    pub max_posts_per_board: Option<Option<i32>>,
    pub last_viewed_at: Option<Option<DateTime<Utc>>>,
}

// ============================================================
// stream_board_subscriptions
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = stream_board_subscriptions)]
pub struct StreamBoardSubscription {
    pub id: Uuid,
    pub stream_id: Uuid,
    pub board_id: Uuid,
    pub include_all_posts: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = stream_board_subscriptions)]
pub struct StreamBoardSubscriptionInsertForm {
    pub stream_id: Uuid,
    pub board_id: Uuid,
    pub include_all_posts: bool,
}

// ============================================================
// stream_flair_subscriptions
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = stream_flair_subscriptions)]
pub struct StreamFlairSubscription {
    pub id: Uuid,
    pub stream_id: Uuid,
    pub board_id: Uuid,
    pub flair_id: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = stream_flair_subscriptions)]
pub struct StreamFlairSubscriptionInsertForm {
    pub stream_id: Uuid,
    pub board_id: Uuid,
    pub flair_id: i32,
}

// ============================================================
// stream_followers
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = stream_followers)]
pub struct StreamFollower {
    pub id: Uuid,
    pub stream_id: Uuid,
    pub user_id: Uuid,
    pub added_to_navbar: bool,
    pub navbar_position: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = stream_followers)]
pub struct StreamFollowerInsertForm {
    pub stream_id: Uuid,
    pub user_id: Uuid,
    pub added_to_navbar: bool,
    pub navbar_position: Option<i32>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = stream_followers)]
pub struct StreamFollowerUpdateForm {
    pub added_to_navbar: Option<bool>,
    pub navbar_position: Option<Option<i32>>,
}

// ============================================================
// stream_excluded_boards
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = stream_excluded_boards)]
pub struct StreamExcludedBoard {
    pub id: Uuid,
    pub stream_id: Uuid,
    pub board_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = stream_excluded_boards)]
pub struct StreamExcludedBoardInsertForm {
    pub stream_id: Uuid,
    pub board_id: Uuid,
}

// ============================================================
// stream_excluded_users
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = stream_excluded_users)]
pub struct StreamExcludedUser {
    pub id: Uuid,
    pub stream_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = stream_excluded_users)]
pub struct StreamExcludedUserInsertForm {
    pub stream_id: Uuid,
    pub user_id: Uuid,
}

// ============================================================
// stream_tags
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = stream_tags)]
pub struct StreamTag {
    pub id: Uuid,
    pub stream_id: Uuid,
    pub tag: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = stream_tags)]
pub struct StreamTagInsertForm {
    pub stream_id: Uuid,
    pub tag: String,
}

// ============================================================
// stream_view_history
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = stream_view_history)]
pub struct StreamViewHistory {
    pub id: Uuid,
    pub stream_id: Uuid,
    pub user_id: Option<Uuid>,
    pub viewed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = stream_view_history)]
pub struct StreamViewHistoryInsertForm {
    pub stream_id: Uuid,
    pub user_id: Option<Uuid>,
}
