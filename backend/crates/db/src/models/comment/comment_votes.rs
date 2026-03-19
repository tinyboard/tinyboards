use crate::schema::comment_votes;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Queryable struct for the comment_votes table.
/// Field order matches schema.rs column order exactly.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = comment_votes)]
pub struct CommentVote {
    pub id: Uuid,
    pub user_id: Uuid,
    pub comment_id: Uuid,
    pub post_id: Uuid,
    pub score: i16,
    pub created_at: DateTime<Utc>,
}

/// Insert form for creating a new comment vote.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = comment_votes)]
pub struct CommentVoteInsertForm {
    pub id: Uuid,
    pub user_id: Uuid,
    pub comment_id: Uuid,
    pub post_id: Uuid,
    pub score: i16,
}

/// Update form for modifying an existing comment vote (score change).
#[derive(Debug, Clone, AsChangeset, Default)]
#[diesel(table_name = comment_votes)]
pub struct CommentVoteUpdateForm {
    pub score: Option<i16>,
}
