use crate::schema::users;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct User {
    id: i32,
    username: String,
    email: Option<String>,
    passhash: String,
    created_utc: i64,
    admin_level: i16,
    is_activated: bool,
    over_18: bool,
    creation_ip: String,
    bio: String,
    bio_html: String,
    referred_by: Option<i32>,
    is_banned: bool,
    unban_utc: i64,
    ban_reason: String,
    defaultsorting: Option<String>,
    defaulttime: Option<String>,
    feed_nonce: i64,
    login_nonce: i64,
    title_id: Option<i32>,
    has_profile: bool,
    has_banner: bool,
    reserved: Option<String>,
    is_nsfw: bool,
    tos_agreed_utc: i64,
    profile_nonce: i64,
    banner_nonce: i64,
    mfa_secret: Option<String>,
    hide_offensive: bool,
    hide_bot: bool,
    show_nsfl: bool,
    is_private: bool,
    is_deleted: bool,
    delete_reason: String,
    filter_nsfw: bool,
    stored_karma: i32,
    stored_subscriber_count: i32,
    auto_join_chat: bool,
    is_nofollow: bool,
    custom_filter_list: String,
    discord_id: Option<String>,
    creation_region: Option<String>,
    ban_evade: i32,
    profile_upload_ip: String,
    banner_upload_ip: String,
    profile_upload_region: String,
    banner_upload_region: String,
    color: String,
    secondary_color: String,
    comment_signature: String,
    comment_signature_html: String,
    profile_set_utc: i64,
    bannner_set_utc: i64,
    original_username: String,
    name_changed_utc: i64,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = users)]
/**
 * not sure if these are all the fields required to insert into the users table, might need more
 */
pub struct InsertUser {
    pub username: String,
    pub email: Option<String>,
    pub passhash: String,
    pub created_utc: i64,
}
