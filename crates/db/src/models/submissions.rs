use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::schema::submissions;

#[derive(Queryable, Serialize, Deserialize)]
pub struct Submissions {
    id: i64,
    title: String,
    post_url: Option<String>,
    body: String,
    body_html: String,
    ban_reason: String,
    embed_url: String,
    meta_title: String,
    meta_description: String,
    author_id: i32,
    repost_id: i32,
    edited_utc: i64,
    created_utc: i64,
    is_banned: bool,
    deleted_utc: i64,
    purged_utc: i64,
    distinguish_level: i16,
    gm_distinguish: i16,
    created_str: Option<String>,
    stickied: bool,
    domain_ref: i32,
    is_approved: i32,
    approved_utc: i64,
    board_id: i32,
    original_board_id: i32,
    over_18: bool,
    creation_ip: String,
    mod_approved: Option<i32>,
    accepted_utc: i64,
    has_thumb: bool,
    post_public: bool,
    is_offensive: bool,
    is_nsfl: bool,
    is_pinned: bool,
    is_bot: bool,
    upvotes: i32,
    downvotes: i32,
    creation_region: Option<String>,
    app_id: Option<i32>,
}

#[derive(Insertable, Serialize, Deserialize, PartialEq)]
#[diesel(table_name = submissions)]
pub struct InsertSubmission {
    pub title: String,
    pub post_url: Option<String>,
    pub body: String,
    pub created_utc: i64,
    pub author_id: i32
}