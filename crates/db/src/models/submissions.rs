use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Submissions {
    id: i64,
    title: Option<String>,
    post_url: Option<String>,
    body: String,
    body_html: String,
    ban_reason: String,
    embed_url: String,
    meta_title: String,
    meta_description: String,
    author_id: i32,
    repost_id: i32,
    edited_utc: i32,
    created_utc: i32,
    is_banned: bool,
    deleted_utc: i32,
    purged_utc: i32,
    distinguish_level: i16,
    gm_distinguish: i16,
    created_str: Option<String>,
    stickied: bool,
    domain_ref: i32,
    is_approved: i32,
    approved_utc: i32,
    board_id: i32,
    original_board_id: i32,
    over_18: bool,
    creation_ip: String,
    mod_approved: Option<i32>,
    accepted_utc: i32,
    has_thumb: bool,
    post_public: bool,
    score_hot: f64,
    score_disputed: f64,
    score_top: f64,
    score_best: f64,
    score_activity: f64,
    is_offensive: bool,
    is_nsfl: bool,
    is_pinned: bool,
    is_bot: bool,
    upvotes: i32,
    downvotes: i32,
    creation_region: Option<String>,
    app_id: Option<i32>,
}
