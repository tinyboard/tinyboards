use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbRegistrationMode,
    models::site::site::{Site as DbSite, SiteUpdateForm},
    models::user::user::AdminPerms,
    schema::site,
    utils::{get_conn, DbPool},
};

use crate::{helpers::permissions, structs::site::LocalSite};

#[derive(Default)]
pub struct SiteConfig;

#[derive(InputObject)]
pub struct UpdateSiteConfigInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub homepage_banner: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub hover_color: Option<String>,
    pub welcome_message: Option<String>,
    pub legal_information: Option<String>,
    pub default_theme: Option<String>,
    pub application_question: Option<String>,
    pub captcha_enabled: Option<bool>,
    pub captcha_difficulty: Option<String>,
    pub enable_downvotes: Option<bool>,
    pub enable_nsfw: Option<bool>,
    pub enable_nsfw_tagging: Option<bool>,
    pub hide_modlog_mod_names: Option<bool>,
    pub reports_email_admins: Option<bool>,
    pub application_email_admins: Option<bool>,
    pub is_private: Option<bool>,
    pub require_email_verification: Option<bool>,
    pub boards_enabled: Option<bool>,
    pub board_creation_admin_only: Option<bool>,
    pub board_creation_mode: Option<String>,
    pub emoji_enabled: Option<bool>,
    pub board_emojis_enabled: Option<bool>,
    pub word_filter_enabled: Option<bool>,
    pub filtered_words: Option<String>,
    pub link_filter_enabled: Option<bool>,
    pub banned_domains: Option<String>,
    pub registration_mode: Option<String>,
}

#[Object]
impl SiteConfig {
    /// Update site configuration (admin with Config permission).
    pub async fn update_site_config(
        &self,
        ctx: &Context<'_>,
        input: UpdateSiteConfigInput,
    ) -> Result<LocalSite> {
        let _admin = permissions::require_admin_permission(ctx, AdminPerms::Config)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        // Get existing site row
        let existing: DbSite = site::table.first(conn).await
            .map_err(|e| tinyboards_utils::TinyBoardsError::Database(format!("Site not found: {}", e)))?;

        let form = SiteUpdateForm {
            name: input.name,
            description: input.description.map(Some),
            icon: input.icon.map(Some),
            homepage_banner: input.homepage_banner.map(Some),
            primary_color: input.primary_color,
            secondary_color: input.secondary_color,
            hover_color: input.hover_color,
            welcome_message: input.welcome_message.map(Some),
            legal_information: input.legal_information.map(Some),
            default_theme: input.default_theme,
            application_question: input.application_question.map(Some),
            captcha_enabled: input.captcha_enabled,
            captcha_difficulty: input.captcha_difficulty,
            enable_downvotes: input.enable_downvotes,
            enable_nsfw: input.enable_nsfw,
            enable_nsfw_tagging: input.enable_nsfw_tagging,
            hide_modlog_mod_names: input.hide_modlog_mod_names,
            reports_email_admins: input.reports_email_admins,
            application_email_admins: input.application_email_admins,
            is_private: input.is_private,
            require_email_verification: input.require_email_verification,
            boards_enabled: input.boards_enabled,
            board_creation_admin_only: input.board_creation_admin_only,
            board_creation_mode: input.board_creation_mode,
            emoji_enabled: input.emoji_enabled,
            board_emojis_enabled: input.board_emojis_enabled,
            word_filter_enabled: input.word_filter_enabled,
            filtered_words: input.filtered_words.map(Some),
            link_filter_enabled: input.link_filter_enabled,
            banned_domains: input.banned_domains.map(Some),
            // Fields not in the input are left None (unchanged)
            default_post_listing_type: None,
            default_avatar: None,
            registration_mode: input.registration_mode.map(|s| match s.as_str() {
                "open" => DbRegistrationMode::Open,
                "invite_only" => DbRegistrationMode::InviteOnly,
                "application_required" | "application" => DbRegistrationMode::ApplicationRequired,
                "closed" => DbRegistrationMode::Closed,
                _ => DbRegistrationMode::Open,
            }),
            is_site_setup: None,
            trusted_user_min_reputation: None,
            trusted_user_min_account_age_days: None,
            trusted_user_manual_approval: None,
            trusted_user_min_posts: None,
            allowed_post_types: None,
            word_filter_applies_to_posts: None,
            word_filter_applies_to_comments: None,
            word_filter_applies_to_usernames: None,
            approved_image_hosts: None,
            image_embed_hosts_only: None,
            max_emojis_per_post: None,
            max_emojis_per_comment: None,
            emoji_max_file_size_mb: None,
        };

        let updated: DbSite = diesel::update(site::table.find(existing.id))
            .set(&form)
            .get_result(conn)
            .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::Database(e.to_string()))?;

        Ok(LocalSite::from(updated))
    }
}
