use crate::schema::stream_board_subscriptions;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Stream subscription to entire boards (all posts regardless of flair)
/// This is the "broad" subscription that includes all content from a board
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = stream_board_subscriptions)]
pub struct StreamBoardSubscription {
    pub id: i32,
    pub stream_id: i32,
    pub board_id: i32,
    pub include_all_posts: bool,
    pub creation_date: NaiveDateTime,
}

/// Form for creating a board subscription
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = stream_board_subscriptions)]
pub struct StreamBoardSubscriptionForm {
    pub stream_id: i32,
    pub board_id: i32,
    pub include_all_posts: bool,
}

/// Form for batch creating board subscriptions
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct BatchBoardSubscriptionForm {
    pub stream_id: i32,
    pub subscriptions: Vec<BoardSubscriptionItem>,
}

/// Individual board subscription item for batch operations
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct BoardSubscriptionItem {
    pub board_id: i32,
    pub include_all_posts: bool,
}

impl Default for StreamBoardSubscriptionForm {
    fn default() -> Self {
        Self {
            stream_id: 0,
            board_id: 0,
            include_all_posts: true,
        }
    }
}
