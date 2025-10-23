use crate::schema::stream_followers;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Stream follower relationship - users following streams
/// Includes navbar customization options
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = stream_followers)]
pub struct StreamFollower {
    pub id: i32,
    pub stream_id: i32,
    pub user_id: i32,
    pub followed_at: NaiveDateTime,
    pub added_to_navbar: bool,
    pub navbar_position: Option<i32>,
}

/// Form for creating a stream follower relationship
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = stream_followers)]
pub struct StreamFollowerForm {
    pub stream_id: i32,
    pub user_id: i32,
    pub added_to_navbar: bool,
    pub navbar_position: Option<i32>,
}

/// Form for updating stream follower settings (navbar)
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = stream_followers)]
pub struct UpdateStreamFollowerForm {
    pub added_to_navbar: Option<bool>,
    pub navbar_position: Option<Option<i32>>,
}

impl Default for StreamFollowerForm {
    fn default() -> Self {
        Self {
            stream_id: 0,
            user_id: 0,
            added_to_navbar: false,
            navbar_position: None,
        }
    }
}

/// Navbar configuration for a user's stream collection
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct NavbarStreamConfig {
    pub stream_id: i32,
    pub stream_name: String,
    pub stream_slug: String,
    pub position: i32,
}
