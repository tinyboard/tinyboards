use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Submissions {
    id: i32,
    title: Nullable<String>,
    post_url: Nullable<String>,
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
    is_banned: Bool,
    deleted_utc: i32,
    purged_utc: i32,
    distinguish_level: i16,
    gm_distinguish: i16,
    created_str: Nullable<String>,
    stickied: Bool,
    domain_ref: i32,
    is_approved: i32,
    approved_utc: i32,
    board_id: i32, 
    original_board_id: i32,
    over_18: Bool,
    creation_ip: String,
    mod_approved: Nullable<i32>,
    accepted_utc: i32,
    has_thumb: Bool,
    post_public: Bool,
    score_hot: BigDecimal,
    score_disputed: BigDecimal,
    score_top: BigDecimal,
    score_best: BigDecimal,
    score_activity: BigDecimal,
    is_offensive: Bool,
    is_nsfl: Bool,
    is_pinned: Bool,
    is_bot: Bool,
    upvotes: i32,
    downvotes: i32,
    creation_region: Nullable<String>,
    app_id: Nullable<i32>,
}