use crate::{schema::site, BoardCreationMode, RegistrationMode};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = site)]
pub struct Site {
    pub id: i32,
    pub site_setup: bool,
    pub invite_only: bool,
    pub enable_downvotes: bool,
    pub open_registration: bool,
    pub enable_nsfw: bool,
    pub board_creation_admin_only: bool,
    pub require_email_verification: bool,
    pub require_application: bool,
    pub application_question: Option<String>,
    pub private_instance: bool,
    pub default_theme: String,
    pub default_post_listing_type: String,
    pub default_avatar: Option<String>,
    pub legal_information: Option<String>,
    pub hide_modlog_mod_names: bool,
    pub application_email_admins: bool,
    pub captcha_enabled: bool,
    pub captcha_difficulty: String,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub reports_email_admins: bool,
    pub name: String,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub hover_color: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub welcome_message: Option<String>,
    pub boards_enabled: bool,
    pub board_creation_mode: String,
    pub trusted_user_min_reputation: i32,
    pub trusted_user_min_account_age_days: i32,
    pub trusted_user_manual_approval: bool,
    pub trusted_user_min_posts: i32,
    pub allowed_post_types: Option<String>,
    pub enable_nsfw_tagging: Option<bool>,
    pub word_filter_enabled: Option<bool>,
    pub filtered_words: Option<String>,
    pub word_filter_applies_to_posts: Option<bool>,
    pub word_filter_applies_to_comments: Option<bool>,
    pub word_filter_applies_to_usernames: Option<bool>,
    pub link_filter_enabled: Option<bool>,
    pub banned_domains: Option<String>,
    pub approved_image_hosts: Option<String>,
    pub image_embed_hosts_only: Option<bool>,
    pub registration_mode: String,
    pub emoji_enabled: bool,
    pub max_emojis_per_post: Option<i32>,
    pub max_emojis_per_comment: Option<i32>,
    pub emoji_max_file_size_mb: i32,
    pub board_emojis_enabled: bool,
    pub homepage_banner: Option<String>,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = site)]
pub struct SiteForm {
    pub site_setup: Option<bool>,
    pub invite_only: Option<bool>,
    pub enable_downvotes: Option<bool>,
    pub open_registration: Option<bool>,
    pub enable_nsfw: Option<bool>,
    pub board_creation_admin_only: Option<bool>,
    pub require_email_verification: Option<bool>,
    pub require_application: Option<bool>,
    pub application_question: Option<Option<String>>,
    pub private_instance: Option<bool>,
    pub default_theme: Option<String>,
    pub default_post_listing_type: Option<String>,
    pub default_avatar: Option<Option<String>>,
    pub legal_information: Option<Option<String>>,
    pub hide_modlog_mod_names: Option<bool>,
    pub application_email_admins: Option<bool>,
    pub captcha_enabled: Option<bool>,
    pub captcha_difficulty: Option<String>,
    pub updated: Option<NaiveDateTime>,
    pub reports_email_admins: Option<bool>,
    pub name: Option<String>,
    pub primary_color: Option<Option<String>>,
    pub secondary_color: Option<Option<String>>,
    pub hover_color: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub icon: Option<Option<String>>,
    pub welcome_message: Option<String>,
    pub boards_enabled: Option<bool>,
    pub board_creation_mode: Option<String>,
    pub trusted_user_min_reputation: Option<i32>,
    pub trusted_user_min_account_age_days: Option<i32>,
    pub trusted_user_manual_approval: Option<bool>,
    pub trusted_user_min_posts: Option<i32>,
    pub allowed_post_types: Option<Option<String>>,
    pub enable_nsfw_tagging: Option<bool>,
    pub word_filter_enabled: Option<bool>,
    pub filtered_words: Option<Option<String>>,
    pub word_filter_applies_to_posts: Option<bool>,
    pub word_filter_applies_to_comments: Option<bool>,
    pub word_filter_applies_to_usernames: Option<bool>,
    pub link_filter_enabled: Option<bool>,
    pub banned_domains: Option<Option<String>>,
    pub approved_image_hosts: Option<Option<String>>,
    pub image_embed_hosts_only: Option<bool>,
    pub registration_mode: Option<String>,
    pub emoji_enabled: Option<bool>,
    pub max_emojis_per_post: Option<Option<i32>>,
    pub max_emojis_per_comment: Option<Option<i32>>,
    pub emoji_max_file_size_mb: Option<i32>,
    pub board_emojis_enabled: Option<bool>,
    pub homepage_banner: Option<Option<String>>,
}

impl Site {
    /// Get the board creation mode as an enum
    pub fn get_board_creation_mode(&self) -> BoardCreationMode {
        BoardCreationMode::from_str(&self.board_creation_mode)
            .unwrap_or(BoardCreationMode::AdminOnly)
    }

    /// Get the registration mode as an enum
    pub fn get_registration_mode(&self) -> RegistrationMode {
        RegistrationMode::from_str(&self.registration_mode)
            .unwrap_or(RegistrationMode::Open)
    }
}
