use crate::schema::{reactions, reaction_aggregates, board_reaction_settings};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = reactions)]
pub struct Reaction {
    pub id: i32,
    pub user_id: i32,
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub emoji: String,
    pub score: i32, // -1, 0, or 1
    pub creation_date: NaiveDateTime,
}

#[derive(Clone, Default, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = reactions)]
pub struct ReactionForm {
    pub user_id: i32,
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub emoji: String,
    pub score: i32,
    pub creation_date: Option<NaiveDateTime>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = reaction_aggregates)]
pub struct ReactionAggregate {
    pub id: i32,
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub emoji: String,
    pub count: i32,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = board_reaction_settings)]
pub struct BoardReactionSettings {
    pub id: i32,
    pub board_id: i32,
    pub emoji_weights: serde_json::Value, // JSONB
    pub reactions_enabled: bool,
}

#[derive(Clone, Default, Serialize, Deserialize, Insertable)]
#[diesel(table_name = board_reaction_settings)]
pub struct BoardReactionSettingsForm {
    pub board_id: i32,
    pub emoji_weights: Option<serde_json::Value>,
    pub reactions_enabled: Option<bool>,
}
