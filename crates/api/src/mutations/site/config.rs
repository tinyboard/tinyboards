use async_graphql::*;
use tinyboards_db::{
    models::{
        user::user::AdminPerms,
        site::site::{Site as DbSite, SiteForm},
    },
    utils::{naive_now, DbPool},
};
use tinyboards_utils::TinyBoardsError;

use crate::{structs::site::LocalSite, LoggedInUser, helpers::files::upload::upload_file_opendal, Settings};

#[derive(Default)]
pub struct SiteConfig;

#[derive(InputObject)]
pub struct UpdateSiteConfigInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub hover_color: Option<String>,
    pub welcome_message: Option<String>,
    pub default_theme: Option<String>,
    pub enable_downvotes: Option<bool>,
    pub open_registration: Option<bool>,
    pub enable_nsfw: Option<bool>,
    pub board_creation_admin_only: Option<bool>,
    pub boards_enabled: Option<bool>,
    pub require_email_verification: Option<bool>,
    pub require_application: Option<bool>,
    pub application_question: Option<String>,
    pub private_instance: Option<bool>,
    pub invite_only: Option<bool>,
    pub board_creation_mode: Option<String>,
    pub trusted_user_min_reputation: Option<i32>,
    pub trusted_user_min_account_age_days: Option<i32>,
    pub trusted_user_manual_approval: Option<bool>,
    pub trusted_user_min_posts: Option<i32>,
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
    pub registration_mode: Option<String>,
    pub default_avatar: Option<String>,
    pub homepage_banner: Option<String>,
}

#[Object]
impl SiteConfig {
    /// Update site configuration (admin only)
    pub async fn update_site_config(
        &self,
        ctx: &Context<'_>,
        input: UpdateSiteConfigInput,
        icon_file: Option<Upload>,
        default_avatar_file: Option<Upload>,
        homepage_banner_file: Option<Upload>,
    ) -> Result<LocalSite> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        // Check admin permissions
        if !user.has_permission(AdminPerms::Config) {
            return Err(TinyBoardsError::from_message(403, "Admin permissions required").into());
        }

        let _site = DbSite::read(pool).await?;
        let settings = ctx.data::<Settings>()?.as_ref();

        // Handle file uploads
        let icon_url = match icon_file {
            Some(file) => Some(upload_file_opendal(file, None, user.id, Some(settings.media.max_site_icon_size_mb), ctx).await?.to_string()),
            None => input.icon
        };

        let default_avatar_url = match default_avatar_file {
            Some(file) => Some(upload_file_opendal(file, None, user.id, Some(settings.media.max_avatar_size_mb), ctx).await?.to_string()),
            None => input.default_avatar
        };

        let homepage_banner_url = match homepage_banner_file {
            Some(file) => Some(upload_file_opendal(file, None, user.id, Some(settings.media.max_banner_size_mb), ctx).await?.to_string()),
            None => input.homepage_banner
        };

        let form = SiteForm {
            name: input.name,
            description: input.description.map(Some),
            icon: icon_url.map(Some),
            primary_color: input.primary_color.map(Some),
            secondary_color: input.secondary_color.map(Some),
            hover_color: input.hover_color.map(Some),
            welcome_message: input.welcome_message,
            default_theme: input.default_theme,
            enable_downvotes: input.enable_downvotes,
            open_registration: input.open_registration,
            enable_nsfw: input.enable_nsfw,
            board_creation_admin_only: input.board_creation_admin_only,
            boards_enabled: input.boards_enabled,
            require_email_verification: input.require_email_verification,
            require_application: input.require_application,
            application_question: input.application_question.map(Some),
            private_instance: input.private_instance,
            invite_only: input.invite_only,
            board_creation_mode: input.board_creation_mode,
            trusted_user_min_reputation: input.trusted_user_min_reputation,
            trusted_user_min_account_age_days: input.trusted_user_min_account_age_days,
            trusted_user_manual_approval: input.trusted_user_manual_approval,
            trusted_user_min_posts: input.trusted_user_min_posts,
            allowed_post_types: input.allowed_post_types.map(Some),
            enable_nsfw_tagging: input.enable_nsfw_tagging,
            word_filter_enabled: input.word_filter_enabled,
            filtered_words: input.filtered_words.map(Some),
            word_filter_applies_to_posts: input.word_filter_applies_to_posts,
            word_filter_applies_to_comments: input.word_filter_applies_to_comments,
            word_filter_applies_to_usernames: input.word_filter_applies_to_usernames,
            link_filter_enabled: input.link_filter_enabled,
            banned_domains: input.banned_domains.map(Some),
            approved_image_hosts: input.approved_image_hosts.map(Some),
            image_embed_hosts_only: input.image_embed_hosts_only,
            registration_mode: input.registration_mode,
            default_avatar: default_avatar_url.map(Some),
            homepage_banner: homepage_banner_url.map(Some),
            updated: Some(naive_now()),
            ..SiteForm::default()
        };

        let updated_site = DbSite::update(pool, &form)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update site configuration"))?;

        Ok(LocalSite::from(updated_site))
    }
}