use crate::schema::post_votes;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Queryable struct for the post_votes table.
/// Field order matches schema.rs column order exactly.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = post_votes)]
pub struct PostVote {
    pub id: Uuid,
    pub user_id: Uuid,
    pub post_id: Uuid,
    pub score: i16,
    pub created_at: DateTime<Utc>,
}

/// Insert form for creating a new post vote.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = post_votes)]
pub struct PostVoteInsertForm {
    pub id: Uuid,
    pub user_id: Uuid,
    pub post_id: Uuid,
    pub score: i16,
}

/// Update form for modifying an existing post vote (score change).
#[derive(Debug, Clone, AsChangeset, Default)]
#[diesel(table_name = post_votes)]
pub struct PostVoteUpdateForm {
    pub score: Option<i16>,
}
