use crate::schema::stream_flair_subscriptions;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Stream subscription to specific flairs within boards
/// This allows a stream to aggregate posts that have specific flair tags
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = stream_flair_subscriptions)]
pub struct StreamFlairSubscription {
    pub id: i32,
    pub stream_id: i32,
    pub board_id: i32,
    pub flair_id: i32,
    pub creation_date: NaiveDateTime,
}

/// Form for creating a flair subscription
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = stream_flair_subscriptions)]
pub struct StreamFlairSubscriptionForm {
    pub stream_id: i32,
    pub board_id: i32,
    pub flair_id: i32,
}

/// Form for batch creating flair subscriptions
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct BatchFlairSubscriptionForm {
    pub stream_id: i32,
    pub subscriptions: Vec<FlairSubscriptionItem>,
}

/// Individual flair subscription item for batch operations
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct FlairSubscriptionItem {
    pub board_id: i32,
    pub flair_id: i32,
}
