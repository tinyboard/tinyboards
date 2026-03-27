use async_graphql::*;
use crate::helpers::permissions;
use tinyboards_db::models::site::site::Site as DbSite;
use tinyboards_db::models::aggregates::SiteAggregates as DbSiteAggregates;
use uuid::Uuid;

#[derive(SimpleObject)]
pub struct SiteStats {
    pub users: i64,
    pub posts: i64,
    pub comments: i64,
    pub boards: i64,
    pub users_active_day: i64,
    pub users_active_week: i64,
    pub users_active_month: i64,
    pub users_active_half_year: i64,
    pub upvotes: i64,
    pub downvotes: i64,
}

impl From<DbSiteAggregates> for SiteStats {
    fn from(v: DbSiteAggregates) -> Self {
        Self {
            users: v.users,
            posts: v.posts,
            comments: v.comments,
            boards: v.boards,
            users_active_day: v.users_active_day,
            users_active_week: v.users_active_week,
            users_active_month: v.users_active_month,
            users_active_half_year: v.users_active_half_year,
            upvotes: v.upvotes,
            downvotes: v.downvotes,
        }
    }
}

/// Public site configuration.
#[derive(SimpleObject)]
#[graphql(complex)]
pub struct LocalSite {
    pub id: ID,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub homepage_banner: Option<String>,
    pub primary_color: String,
    pub secondary_color: String,
    pub hover_color: String,
    pub legal_information: Option<String>,
    pub default_theme: String,
    pub default_post_listing_type: String,
    pub default_avatar: Option<String>,
    pub registration_mode: String,
    pub application_question: Option<String>,
    pub is_site_setup: bool,
    pub is_private: bool,
    pub require_email_verification: bool,
    pub application_email_admins: bool,
    pub captcha_enabled: bool,
    pub captcha_difficulty: String,
    pub enable_downvotes: bool,
    #[graphql(name = "enableNSFW")]
    pub enable_nsfw: bool,
    #[graphql(name = "enableNSFWTagging")]
    pub enable_nsfw_tagging: bool,
    pub hide_modlog_mod_names: bool,
    pub reports_email_admins: bool,
    pub boards_enabled: bool,
    pub board_creation_admin_only: bool,
    pub board_creation_mode: String,
    pub trusted_user_min_reputation: i32,
    pub trusted_user_min_account_age_days: i32,
    pub trusted_user_manual_approval: bool,
    pub trusted_user_min_posts: i32,
    pub allowed_post_types: Option<String>,
    pub word_filter_enabled: bool,
    pub filtered_words: Option<String>,
    pub word_filter_applies_to_posts: bool,
    pub word_filter_applies_to_comments: bool,
    pub word_filter_applies_to_usernames: bool,
    pub link_filter_enabled: bool,
    pub banned_domains: Option<String>,
    pub approved_image_hosts: Option<String>,
    pub image_embed_hosts_only: bool,
    pub emoji_enabled: bool,
    pub max_emojis_per_post: Option<i32>,
    pub max_emojis_per_comment: Option<i32>,
    pub emoji_max_file_size_mb: i32,
    pub board_emojis_enabled: bool,
    pub custom_css_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
    // Hidden fields — only admins can see
    #[graphql(skip)]
    pub welcome_message_: Option<String>,
    #[graphql(skip)]
    pub custom_css_: Option<String>,
}

#[ComplexObject]
impl LocalSite {
    pub async fn welcome_message(&self, ctx: &Context<'_>) -> Option<String> {
        // Only admins with Config permission can see the welcome message
        let user = permissions::optional_auth(ctx);
        match user {
            Some(u) if u.has_permission(tinyboards_db::models::user::user::AdminPerms::Config) => {
                self.welcome_message_.clone()
            }
            _ => None,
        }
    }

    /// Custom CSS for the site. Returned to all users when custom_css_enabled is true,
    /// so the frontend can inject it into a <style> tag. Admins always see it (for editing).
    pub async fn custom_css(&self, ctx: &Context<'_>) -> Option<String> {
        if self.custom_css_enabled {
            return self.custom_css_.clone();
        }
        // If disabled, only admins can see it (so they can edit it before enabling)
        let user = permissions::optional_auth(ctx);
        match user {
            Some(u) if u.has_permission(tinyboards_db::models::user::user::AdminPerms::Config) => {
                self.custom_css_.clone()
            }
            _ => None,
        }
    }
}

impl From<DbSite> for LocalSite {
    fn from(v: DbSite) -> Self {
        Self {
            id: ID(v.id.to_string()),
            name: v.name,
            description: v.description,
            icon: v.icon,
            homepage_banner: v.homepage_banner,
            primary_color: v.primary_color,
            secondary_color: v.secondary_color,
            hover_color: v.hover_color,
            legal_information: v.legal_information,
            default_theme: v.default_theme,
            default_post_listing_type: match v.default_post_listing_type {
                tinyboards_db::enums::DbListingType::All => "all".to_string(),
                tinyboards_db::enums::DbListingType::Subscribed => "subscribed".to_string(),
                tinyboards_db::enums::DbListingType::Local => "local".to_string(),
            },
            default_avatar: v.default_avatar,
            registration_mode: match v.registration_mode {
                tinyboards_db::enums::DbRegistrationMode::Open => "open".to_string(),
                tinyboards_db::enums::DbRegistrationMode::InviteOnly => "invite_only".to_string(),
                tinyboards_db::enums::DbRegistrationMode::ApplicationRequired => "application_required".to_string(),
                tinyboards_db::enums::DbRegistrationMode::Closed => "closed".to_string(),
            },
            application_question: v.application_question,
            is_site_setup: v.is_site_setup,
            is_private: v.is_private,
            require_email_verification: v.require_email_verification,
            application_email_admins: v.application_email_admins,
            captcha_enabled: v.captcha_enabled,
            captcha_difficulty: v.captcha_difficulty,
            enable_downvotes: v.enable_downvotes,
            enable_nsfw: v.enable_nsfw,
            enable_nsfw_tagging: v.enable_nsfw_tagging,
            hide_modlog_mod_names: v.hide_modlog_mod_names,
            reports_email_admins: v.reports_email_admins,
            boards_enabled: v.boards_enabled,
            board_creation_admin_only: v.board_creation_admin_only,
            board_creation_mode: v.board_creation_mode,
            trusted_user_min_reputation: v.trusted_user_min_reputation,
            trusted_user_min_account_age_days: v.trusted_user_min_account_age_days,
            trusted_user_manual_approval: v.trusted_user_manual_approval,
            trusted_user_min_posts: v.trusted_user_min_posts,
            allowed_post_types: v.allowed_post_types,
            word_filter_enabled: v.word_filter_enabled,
            filtered_words: v.filtered_words,
            word_filter_applies_to_posts: v.word_filter_applies_to_posts,
            word_filter_applies_to_comments: v.word_filter_applies_to_comments,
            word_filter_applies_to_usernames: v.word_filter_applies_to_usernames,
            link_filter_enabled: v.link_filter_enabled,
            banned_domains: v.banned_domains,
            approved_image_hosts: v.approved_image_hosts,
            image_embed_hosts_only: v.image_embed_hosts_only,
            emoji_enabled: v.emoji_enabled,
            max_emojis_per_post: v.max_emojis_per_post,
            max_emojis_per_comment: v.max_emojis_per_comment,
            emoji_max_file_size_mb: v.emoji_max_file_size_mb,
            board_emojis_enabled: v.board_emojis_enabled,
            custom_css_enabled: v.custom_css_enabled,
            created_at: v.created_at.to_rfc3339(),
            updated_at: v.updated_at.to_rfc3339(),
            welcome_message_: v.welcome_message,
            custom_css_: v.custom_css,
        }
    }
}
