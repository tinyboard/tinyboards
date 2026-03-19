use crate::schema::languages;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A language available for content tagging (posts, comments, boards).
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = languages)]
pub struct Language {
    pub id: i32,
    pub code: String,
    pub name: String,
}
