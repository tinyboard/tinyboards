use async_graphql::*;
use serde::{Deserialize, Serialize};
use tinyboards_db::models::reaction::reactions::{
    Reaction as DbReaction,
    ReactionAggregate as DbReactionAggregate,
    BoardReactionSettings as DbBoardReactionSettings,
};

/// A user's reaction to a post or comment
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct Reaction {
    pub id: i32,
    pub user_id: i32,
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub emoji: String,
    pub score: i32, // -1 (negative), 0 (neutral), or 1 (positive)
    pub creation_date: String,
}

#[ComplexObject]
impl Reaction {
    /// The user who created this reaction
    async fn creator(&self, ctx: &Context<'_>) -> Result<super::user::User> {
        use tinyboards_db::{models::user::user::User, traits::Crud, utils::DbPool};
        let pool = ctx.data::<DbPool>()?;
        let user = User::read(pool, self.user_id).await?;
        Ok(super::user::User::from(user))
    }
}

impl From<DbReaction> for Reaction {
    fn from(r: DbReaction) -> Self {
        Reaction {
            id: r.id,
            user_id: r.user_id,
            post_id: r.post_id,
            comment_id: r.comment_id,
            emoji: r.emoji,
            score: r.score,
            creation_date: r.creation_date.to_string(),
        }
    }
}

/// Aggregated reaction counts for an emoji
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct ReactionAggregate {
    pub id: i32,
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub emoji: String,
    pub count: i32,
}

#[ComplexObject]
impl ReactionAggregate {
    /// List of users who reacted with this emoji
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<super::user::User>> {
        use tinyboards_db::{models::reaction::reactions::Reaction, utils::DbPool};

        let pool = ctx.data::<DbPool>()?;

        // Get all reactions for this emoji on this post/comment
        let reactions = if let Some(post_id) = self.post_id {
            Reaction::list_for_post(pool, post_id).await?
        } else if let Some(comment_id) = self.comment_id {
            Reaction::list_for_comment(pool, comment_id).await?
        } else {
            return Ok(vec![]);
        };

        // Filter to only reactions matching this emoji and get unique users
        let user_ids: Vec<i32> = reactions
            .iter()
            .filter(|r| r.emoji == self.emoji)
            .map(|r| r.user_id)
            .collect();

        // Fetch user details
        use tinyboards_db::{models::user::user::User, traits::Crud};
        let mut users = Vec::new();
        for user_id in user_ids {
            if let Ok(user) = User::read(pool, user_id).await {
                users.push(super::user::User::from(user));
            }
        }

        Ok(users)
    }
}

impl From<DbReactionAggregate> for ReactionAggregate {
    fn from(r: DbReactionAggregate) -> Self {
        ReactionAggregate {
            id: r.id,
            post_id: r.post_id,
            comment_id: r.comment_id,
            emoji: r.emoji,
            count: r.count,
        }
    }
}

/// Board reaction settings
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct BoardReactionSettings {
    pub id: i32,
    pub board_id: i32,
    pub emoji_weights: Json<serde_json::Value>, // JSONB mapping emoji -> score (-1, 0, or 1)
    pub reactions_enabled: bool,
}

impl From<DbBoardReactionSettings> for BoardReactionSettings {
    fn from(s: DbBoardReactionSettings) -> Self {
        BoardReactionSettings {
            id: s.id,
            board_id: s.board_id,
            emoji_weights: Json(s.emoji_weights),
            reactions_enabled: s.reactions_enabled,
        }
    }
}
