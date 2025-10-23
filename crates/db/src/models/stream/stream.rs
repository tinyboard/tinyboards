use crate::schema::streams;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Main Stream model representing a custom feed
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = streams)]
pub struct Stream {
    pub id: i32,
    pub creator_id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_public: bool,
    pub is_discoverable: bool,
    pub share_token: Option<String>,
    pub sort_type: String,
    pub time_range: Option<String>,
    pub show_nsfw: bool,
    pub max_posts_per_board: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub last_viewed_at: Option<NaiveDateTime>,
}

/// Form for creating a new stream
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = streams)]
pub struct CreateStreamForm {
    pub creator_id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_public: bool,
    pub is_discoverable: bool,
    pub share_token: Option<String>,
    pub sort_type: String,
    pub time_range: Option<String>,
    pub show_nsfw: bool,
    pub max_posts_per_board: Option<i32>,
}

/// Form for updating an existing stream
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, AsChangeset)]
#[diesel(table_name = streams)]
pub struct UpdateStreamForm {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<Option<String>>,
    pub icon: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub is_public: Option<bool>,
    pub is_discoverable: Option<bool>,
    pub share_token: Option<Option<String>>,
    pub sort_type: Option<String>,
    pub time_range: Option<Option<String>>,
    pub show_nsfw: Option<bool>,
    pub max_posts_per_board: Option<Option<i32>>,
    pub updated_at: Option<Option<NaiveDateTime>>,
    pub last_viewed_at: Option<Option<NaiveDateTime>>,
}

/// Stream visibility enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamVisibility {
    Public,
    Private,
    ShareLink,
}

impl StreamVisibility {
    pub fn from_stream(stream: &Stream) -> Self {
        if stream.is_public {
            StreamVisibility::Public
        } else if stream.share_token.is_some() {
            StreamVisibility::ShareLink
        } else {
            StreamVisibility::Private
        }
    }
}

/// Sort type specifically for streams (defaults)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamSortType {
    Hot,
    New,
    Top,
    Active,
}

impl StreamSortType {
    pub fn to_string(&self) -> String {
        match self {
            StreamSortType::Hot => "hot".to_string(),
            StreamSortType::New => "new".to_string(),
            StreamSortType::Top => "top".to_string(),
            StreamSortType::Active => "active".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "hot" => StreamSortType::Hot,
            "new" => StreamSortType::New,
            "top" => StreamSortType::Top,
            "active" => StreamSortType::Active,
            _ => StreamSortType::Hot,
        }
    }
}

impl Default for StreamSortType {
    fn default() -> Self {
        StreamSortType::Hot
    }
}

/// Time range for Top sorting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamTimeRange {
    Day,
    Week,
    Month,
    Year,
    All,
}

impl StreamTimeRange {
    pub fn to_string(&self) -> String {
        match self {
            StreamTimeRange::Day => "day".to_string(),
            StreamTimeRange::Week => "week".to_string(),
            StreamTimeRange::Month => "month".to_string(),
            StreamTimeRange::Year => "year".to_string(),
            StreamTimeRange::All => "all".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "day" => StreamTimeRange::Day,
            "week" => StreamTimeRange::Week,
            "month" => StreamTimeRange::Month,
            "year" => StreamTimeRange::Year,
            "all" => StreamTimeRange::All,
            _ => StreamTimeRange::All,
        }
    }
}

impl Default for StreamTimeRange {
    fn default() -> Self {
        StreamTimeRange::All
    }
}

/// Safe representation of a stream (for public queries)
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = streams)]
pub struct StreamSafe {
    pub id: i32,
    pub creator_id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub default_sort: String,
    pub default_time_range: String,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}
