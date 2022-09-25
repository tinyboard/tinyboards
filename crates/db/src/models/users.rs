use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Users {
    id: i32,
    username: String,
    email: String,
    passhash: String,
    created_utc: i32,
    admin_level: i16,
    is_activated: Bool,
    over_18: Bool,
    creation_ip: String,
    bio: String,
    bio_html: String,
    referred_by: Nullable<i32>,
    is_banned: Bool,
    unban_utc: i32,
    ban_reason: String,
    defaultsorting: Nullable<String>,
    defaulttime: Nullable<String>,
    feed_nonce: i32,
    login_nonce: i32,
    title_id: Nullable<i32>,
    has_profile: Bool,
    has_banner: Bool,
    reserved: Nullable<String>,
    is_nsfw: Bool,
    tos_agreed_utc: i32,
    profile_nonce: i32,
    banner_nonce: i32,
    mfa_secret: Nullable<String>,
    hide_offensive: Bool,
    hide_bot: Bool,
    show_nsfl: Bool,
    is_private: Bool,
    is_deleted: Bool,
    delete_reason: String,
    filter_nsfw: Bool,
    stored_karma: i32,
    stored_subscriber_count: i32,
    auto_join_chat: Bool,
    is_nofollow: Bool,
    custom_filter_list: String,
    discord_id: Nullable<String>,
    creation_region: Nullable<String>,
    ban_evade: i32,
    profile_upload_ip: String,
    banner_upload_ip: String,
    profile_upload_region: String,
    banner_upload_region: String,
    color: String,
    secondary_color: String,
    comment_signature: String,
    comment_signature_html: String,
    profile_set_utc: i32,
    bannner_set_utc: i32,
    original_username: String,
    name_changed_utc: i32,
}