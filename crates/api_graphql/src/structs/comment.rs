use std::collections::HashMap;

use async_graphql::*;
use tinyboards_db::{
    aggregates::structs::CommentAggregates as DbCommentAggregates,
    models::comment::comments::Comment as DbComment,
};

use crate::Censorable;

#[derive(SimpleObject)]
pub struct Comment {
    id: i32,
    creator_id: i32,
    post_id: i32,
    parent_id: Option<i32>,
    body: String,
    body_html: String,
    is_removed: bool,
    is_locked: bool,
    is_deleted: bool,
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
