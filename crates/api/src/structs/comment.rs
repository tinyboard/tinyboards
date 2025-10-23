use crate::PostgresLoader;

use async_graphql::*;
use dataloader::DataLoader;
use tinyboards_db::{
    aggregates::structs::CommentAggregates as DbCommentAggregates,
    models::comment::comments::Comment as DbComment,
};

use crate::{
    newtypes::{BoardId, UserId, PostIdForComment, SavedForCommentId, VoteForCommentId},
    structs::{boards::Board, user::User},
    Censorable,
};

use super::post::Post;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Comment {
    id: i32,
    creator_id: i32,
    post_id: i32,
    parent_id: Option<i32>,
    body: String,
    #[graphql(name = "bodyHTML")]
    body_html: String,
    pub(crate) is_removed: bool,
    is_locked: bool,
    pub(crate) is_active: bool,
    is_pinned: bool,
    creation_date: String,
    level: i32,
    updated: Option<String>,
    board_id: i32,
    local: bool,
    creator_vote: i32,
    quoted_comment_id: Option<i32>,
    slug: String,
    replies: Option<Vec<Self>>,
    #[graphql(skip)]
    counts: DbCommentAggregates,
}

#[ComplexObject]
impl Comment {
    pub async fn score(&self) -> i64 {
        self.counts.score
    }

    pub async fn upvotes(&self) -> i64 {
        self.counts.upvotes
    }

    pub async fn downvotes(&self) -> i64 {
        self.counts.downvotes
    }

    pub async fn reply_count(&self) -> i32 {
        self.counts.child_count
    }

    pub async fn creator(&self, ctx: &Context<'_>) -> Result<Option<User>> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(UserId(self.creator_id))
            .await
            .map_err(|e| e.into())
    }

    pub async fn board(&self, ctx: &Context<'_>) -> Result<Option<Board>> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(BoardId(self.board_id))
            .await
            .map_err(|e| e.into())
    }

    pub async fn post(&self, ctx: &Context<'_>) -> Result<Post> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(PostIdForComment(self.post_id))
            .await
            .map(|post_opt| post_opt.expect(
                &format!("Failed to load post corresponding to post ID {} while loading the parent post of comment with ID {}.", self.post_id, self.id)
            ))
            .map_err(|e| e.into())
    }

    pub async fn my_vote(&self, ctx: &Context<'_>) -> Result<i32> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();

        loader
            .load_one(VoteForCommentId(self.id))
            .await
            .map(|v| v.unwrap_or(0))
            .map_err(|e| e.into())
    }

    pub async fn is_saved(&self, ctx: &Context<'_>) -> Result<bool> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();

        loader
            .load_one(SavedForCommentId(self.id))
            .await
            .map(|v| v.unwrap_or(false))
            .map_err(|e| e.into())
    }

    /// Get aggregated reaction counts for this comment
    pub async fn reaction_counts(&self, ctx: &Context<'_>) -> Result<Vec<super::reaction::ReactionAggregate>> {
        use tinyboards_db::models::reaction::reactions::ReactionAggregate;
        use tinyboards_db::utils::DbPool;
        use tinyboards_utils::TinyBoardsError;
        let pool = ctx.data::<DbPool>()?;

        let aggregates = ReactionAggregate::list_for_comment(pool, self.id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load reaction counts"))?;

        Ok(aggregates.into_iter().map(super::reaction::ReactionAggregate::from).collect())
    }

    /// Get the current user's reaction to this comment (if any)
    pub async fn my_reaction(&self, ctx: &Context<'_>) -> Result<Option<super::reaction::Reaction>> {
        use tinyboards_db::models::reaction::reactions::Reaction;
        use tinyboards_db::utils::DbPool;
        use tinyboards_utils::TinyBoardsError;
        use crate::LoggedInUser;
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.inner();

        if let Some(u) = user {
            // Get all user's reactions for this comment
            let reactions = Reaction::list_for_comment(pool, self.id)
                .await
                .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load reactions"))?;

            // Filter to current user's reactions
            let my_reactions: Vec<_> = reactions.into_iter()
                .filter(|r| r.user_id == u.id)
                .map(super::reaction::Reaction::from)
                .collect();

            // Return first reaction (users can have multiple reactions with different emojis)
            Ok(my_reactions.into_iter().next())
        } else {
            Ok(None)
        }
    }
}

impl From<(DbComment, DbCommentAggregates)> for Comment {
    fn from((comment, counts): (DbComment, DbCommentAggregates)) -> Self {
        Self {
            id: comment.id,
            creator_id: comment.creator_id,
            post_id: comment.post_id,
            parent_id: comment.parent_id,
            body: comment.body,
            body_html: comment.body_html,
            is_removed: comment.is_removed,
            is_active: comment.is_deleted,
            is_locked: comment.is_locked,
            is_pinned: comment.is_pinned.unwrap_or(false),
            creation_date: comment.creation_date.to_string(),
            level: comment.level,
            updated: comment.updated.map(|u| u.to_string()),
            board_id: comment.board_id,
            local: true,
            creator_vote: comment.creator_vote,
            quoted_comment_id: comment.quoted_comment_id,
            slug: comment.slug,
            counts,
            replies: None,
        }
    }
}

impl Censorable for Comment {
    /// Censor comment body for deleted/removed comments. Used when comments are nested.
    fn censor(&mut self, my_user_id: i32, is_admin: bool, is_mod: bool) {
        // nothing to do here lol
        if !(self.is_removed || self.is_active) {
            return;
        }

        // admins see everything
        if is_admin {
            return;
        }

        // mods can see removed content, and you can see your own removed content
        if self.is_removed && (is_mod || self.creator_id == my_user_id) {
            return;
        }

        let censor_text = if self.is_active {
            "[ deleted by creator ]"
        } else {
            "[ removed by mod or admin ]"
        }
        .to_string();

        self.body = censor_text.clone();
        self.body_html = censor_text;
    }
}
