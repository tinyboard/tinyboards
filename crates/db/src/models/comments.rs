use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::schema::comments;

#[derive(Queryable, Serialize, Deserialize)]
pub struct Comments {
    id: i32,
    body: String,
    body_html: Option<String>,
    ban_reason: String,
    author_id: i32,
    parent_submission: i32,
    created_utc: i64,
    edited_utc: i64,
    is_banned: bool,
    gm_distinguish: i32,
    distinguished_board: Option<i32>,
    deleted_utc: i64,
    purged_utc: i64,
    is_approved: i32,
    approved_utc: i64,
    creation_ip: String,
    comment_level: i32,
    parent_comment_id: i32,
    original_board_id: i32,
    over_18: bool,
    is_offensive: bool,
    is_nsfl: bool,
    is_bot: bool,
    is_pinned: bool,
    creation_region: Option<String>,
    app_id: Option<i32>,
    upvotes: i32,
    downvotes: i32,
}

#[derive(Insertable, Serialize, Deserialize, PartialEq)]
#[diesel(table_name = comments)]
pub struct InsertComment {
    pub author_id: i32,
    pub parent_submission: i32,
    pub body: String,
    pub created_utc: i64,
}
