use async_graphql::*;
use serde::{Deserialize, Serialize};
use tinyboards_db::models::reaction::{
    Reaction as DbReaction,
    BoardReactionSettings as DbBoardReactionSettings,
};

/// A user's reaction to a post or comment
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct Reaction {
    pub id: ID,
    pub user_id: ID,
    pub post_id: Option<ID>,
    pub comment_id: Option<ID>,
    pub emoji: String,
    pub score: i32,
    #[graphql(name = "createdAt")]
    pub created_at: String,
}

impl From<DbReaction> for Reaction {
    fn from(r: DbReaction) -> Self {
        Reaction {
            id: r.id.to_string().into(),
            user_id: r.user_id.to_string().into(),
            post_id: r.post_id.map(|id| id.to_string().into()),
            comment_id: r.comment_id.map(|id| id.to_string().into()),
            emoji: r.emoji,
            score: r.score,
            created_at: r.created_at.to_string(),
        }
    }
}

/// Aggregated reaction count for a specific emoji on a post or comment
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct ReactionAggregate {
    pub id: ID,
    pub post_id: Option<ID>,
    pub comment_id: Option<ID>,
    pub emoji: String,
    pub count: i32,
}

impl From<tinyboards_db::models::aggregates::ReactionAggregates> for ReactionAggregate {
    fn from(a: tinyboards_db::models::aggregates::ReactionAggregates) -> Self {
        ReactionAggregate {
            id: a.id.to_string().into(),
            post_id: a.post_id.map(|id| id.to_string().into()),
            comment_id: a.comment_id.map(|id| id.to_string().into()),
            emoji: a.emoji,
            count: a.count,
        }
    }
}

/// Board reaction settings
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct BoardReactionSettings {
    pub id: ID,
    pub board_id: ID,
    pub emoji_weights: Json<serde_json::Value>,
    #[graphql(name = "reactionsEnabled")]
    pub reactions_enabled: bool,
    #[graphql(name = "reactionEmojis")]
    pub reaction_emojis: Json<serde_json::Value>,
}

impl From<DbBoardReactionSettings> for BoardReactionSettings {
    fn from(s: DbBoardReactionSettings) -> Self {
        BoardReactionSettings {
            id: s.id.to_string().into(),
            board_id: s.board_id.to_string().into(),
            emoji_weights: Json(s.emoji_weights),
            reactions_enabled: s.is_reactions_enabled,
            reaction_emojis: Json(s.reaction_emojis),
        }
    }
}
