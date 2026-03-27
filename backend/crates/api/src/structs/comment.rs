use crate::PostgresLoader;

use async_graphql::*;
use async_graphql::dataloader::DataLoader;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        aggregates::CommentAggregates as DbCommentAggregates,
        comment::comments::Comment as DbComment,
        reaction::Reaction as DbReaction,
    },
    schema::{reactions, reaction_aggregates},
    utils::{DbPool, get_conn},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{
    newtypes::{BoardId, UserId, PostIdForComment, SavedForCommentId, VoteForCommentId},
    structs::{boards::Board, user::User},
    Censorable, LoggedInUser,
};

use super::post::Post;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Comment {
    pub id: ID,
    pub creator_id: ID,
    pub post_id: ID,
    pub parent_id: Option<String>,
    pub body: String,
    #[graphql(name = "bodyHTML")]
    pub body_html: String,
    pub(crate) is_removed: bool,
    pub is_locked: bool,
    pub(crate) is_deleted: bool,
    pub is_pinned: bool,
    pub created_at: String,
    pub level: i32,
    pub updated_at: String,
    pub board_id: ID,
    pub quoted_comment_id: Option<String>,
    pub slug: String,
    pub approval_status: String,
    pub distinguished_as: Option<String>,
    pub replies: Option<Vec<Self>>,
    // Internal UUID fields for dataloaders
    #[graphql(skip)]
    pub(crate) uuid_id: Uuid,
    #[graphql(skip)]
    pub(crate) uuid_creator_id: Uuid,
    #[graphql(skip)]
    pub(crate) uuid_post_id: Uuid,
    #[graphql(skip)]
    pub(crate) uuid_board_id: Uuid,
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
            .load_one(UserId(self.uuid_creator_id))
            .await
            .map_err(|e| e.into())
    }

    pub async fn board(&self, ctx: &Context<'_>) -> Result<Option<Board>> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(BoardId(self.uuid_board_id))
            .await
            .map_err(|e| e.into())
    }

    pub async fn post(&self, ctx: &Context<'_>) -> Result<Post> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(PostIdForComment(self.uuid_post_id))
            .await
            .map(|post_opt| post_opt.expect(
                &format!("Failed to load post for comment {}", self.uuid_id)
            ))
            .map_err(|e| e.into())
    }

    pub async fn my_vote(&self, ctx: &Context<'_>) -> Result<i32> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(VoteForCommentId(self.uuid_id))
            .await
            .map(|v| v.unwrap_or(0))
            .map_err(|e| e.into())
    }

    pub async fn is_saved(&self, ctx: &Context<'_>) -> Result<bool> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(SavedForCommentId(self.uuid_id))
            .await
            .map(|v| v.unwrap_or(false))
            .map_err(|e| e.into())
    }

    /// Get aggregated reaction counts for this comment
    pub async fn reaction_counts(&self, ctx: &Context<'_>) -> Result<Vec<super::reaction::ReactionAggregate>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let aggregates: Vec<tinyboards_db::models::aggregates::ReactionAggregates> =
            reaction_aggregates::table
                .filter(reaction_aggregates::comment_id.eq(self.uuid_id))
                .load(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(aggregates.into_iter().map(super::reaction::ReactionAggregate::from).collect())
    }

    /// Get the current user's reaction to this comment (if any)
    pub async fn my_reaction(&self, ctx: &Context<'_>) -> Result<Option<super::reaction::Reaction>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.inner();

        if let Some(u) = user {
            let conn = &mut get_conn(pool).await?;
            let reaction: Option<DbReaction> = reactions::table
                .filter(reactions::comment_id.eq(self.uuid_id))
                .filter(reactions::user_id.eq(u.id))
                .first(conn)
                .await
                .optional()
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            Ok(reaction.map(super::reaction::Reaction::from))
        } else {
            Ok(None)
        }
    }
}

impl From<(DbComment, DbCommentAggregates)> for Comment {
    fn from((comment, counts): (DbComment, DbCommentAggregates)) -> Self {
        let is_deleted = comment.deleted_at.is_some();
        Self {
            id: ID(comment.id.to_string()),
            creator_id: ID(comment.creator_id.to_string()),
            post_id: ID(comment.post_id.to_string()),
            parent_id: comment.parent_id.map(|id| id.to_string()),
            body: comment.body.clone(),
            body_html: comment.body_html.clone(),
            is_removed: comment.is_removed,
            is_deleted,
            is_locked: comment.is_locked,
            is_pinned: comment.is_pinned,
            created_at: comment.created_at.to_rfc3339(),
            level: comment.level,
            updated_at: comment.updated_at.to_rfc3339(),
            board_id: ID(comment.board_id.to_string()),
            quoted_comment_id: comment.quoted_comment_id.map(|id| id.to_string()),
            slug: comment.slug.clone(),
            approval_status: format!("{:?}", comment.approval_status).to_lowercase(),
            distinguished_as: comment.distinguished_as.clone(),
            uuid_id: comment.id,
            uuid_creator_id: comment.creator_id,
            uuid_post_id: comment.post_id,
            uuid_board_id: comment.board_id,
            counts,
            replies: None,
        }
    }
}

impl Censorable for Comment {
    fn censor(&mut self, my_user_id: uuid::Uuid, is_admin: bool, is_mod: bool) {
        if !(self.is_removed || self.is_deleted) {
            return;
        }

        if is_admin {
            return;
        }

        // mods can see removed content, and you can see your own removed content
        if self.is_removed && (is_mod || self.uuid_creator_id == my_user_id) {
            return;
        }

        let censor_text = if self.is_deleted {
            "[ deleted by creator ]"
        } else {
            "[ removed by mod or admin ]"
        }
        .to_string();

        self.body = censor_text.clone();
        self.body_html = censor_text;
    }
}
