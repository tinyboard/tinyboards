use std::collections::HashMap;

use async_graphql::*;
use dataloader::DataLoader;
use tinyboards_db::{
    aggregates::structs::CommentAggregates as DbCommentAggregates,
    models::comment::comments::Comment as DbComment,
};

use crate::{
    newtypes::{BoardId, PersonId, PostIdForComment, SavedForCommentId, VoteForCommentId},
    structs::{boards::Board, person::Person},
    Censorable, PostgresLoader,
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
    pub(crate) is_deleted: bool,
    is_pinned: bool,
    creation_date: String,
    level: i32,
    updated: Option<String>,
    board_id: i32,
    local: bool,
    replies: Option<Vec<Self>>,
    #[graphql(skip)]
    counts: DbCommentAggregates,
}

/// The tree functions defined here were copied from db_views/comment_view.rs, where these functions are documented in detail.
/// These are the same, but for the Comment GraphQL object instead of CommentView.
impl Comment {
    fn tree_wrap(self, replies_table: &mut HashMap<i32, Vec<Self>>) -> Self {
        Self {
            replies: {
                let mut replies = Vec::new();

                if let Some(children) = replies_table.remove(&self.id) {
                    for child in children.into_iter() {
                        replies.push(child.tree_wrap(replies_table));
                    }
                }

                Some(replies)
            },
            ..self
        }
    }

    /// Order comments into a tree structure.
    pub fn tree(comments: Vec<Self>, top_comment_id: Option<i32>) -> Vec<Self> {
        let mut hash_table = HashMap::new();
        let top_comment_id = top_comment_id.unwrap_or(-1);

        let mut top_level_comments = Vec::new();

        for comment in comments.into_iter() {
            match comment.parent_id {
                Some(parent_id) => {
                    if comment.id == top_comment_id {
                        top_level_comments.push(comment);
                    } else {
                        let replies = hash_table.entry(parent_id).or_insert(Vec::new());
                        replies.push(comment);
                    }
                }
                None => top_level_comments.push(comment),
            }
        }

        let mut tree = Vec::new();

        for comment in top_level_comments.into_iter() {
            tree.push(comment.tree_wrap(&mut hash_table));
        }

        tree
    }
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

    pub async fn reply_count(&self) -> Option<i32> {
        self.counts.reply_count
    }

    pub async fn creator(&self, ctx: &Context<'_>) -> Result<Option<Person>> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(PersonId(self.creator_id))
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

    pub async fn post(&self, ctx: &Context<'_>) -> Result<Option<Post>> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(PostIdForComment(self.post_id))
            .await
            .map_err(|e| e.into())
    }

    pub async fn my_vote(&self, ctx: &Context<'_>) -> Result<i16> {
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
            is_deleted: comment.is_deleted,
            is_locked: comment.is_locked,
            is_pinned: comment.is_pinned.unwrap_or(false),
            creation_date: comment.creation_date.to_string(),
            level: comment.level,
            updated: comment.updated.map(|u| u.to_string()),
            board_id: comment.board_id,
            local: comment.local,
            counts,
            replies: None,
        }
    }
}

impl Censorable for Comment {
    /// Censor comment body for deleted/removed comments. Used when comments are nested.
    fn censor(&mut self, my_person_id: i32, is_admin: bool, is_mod: bool) {
        // nothing to do here lol
        if !(self.is_removed || self.is_deleted) {
            return;
        }

        // admins see everything
        if is_admin {
            return;
        }

        // mods can see removed content, and you can see your own removed content
        if self.is_removed && (is_mod || self.creator_id == my_person_id) {
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
