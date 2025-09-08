use async_graphql::*;
use tinyboards_db::{
    models::{
        person::local_user::AdminPerms,
        site::local_site::{LocalSite as DbLocalSite, LocalSiteForm},
    },
    utils::{naive_now, DbPool},
};
use tinyboards_utils::TinyBoardsError;

use crate::{structs::local_site::LocalSite, LoggedInUser};

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
    pub require_email_verification: Option<bool>,
    pub require_application: Option<bool>,
    pub application_question: Option<String>,
    pub private_instance: Option<bool>,
    pub invite_only: Option<bool>,
}

#[Object]
impl SiteConfig {
    /// Update site configuration (admin only)
    pub async fn update_site_config(
        &self,
        ctx: &Context<'_>,
        input: UpdateSiteConfigInput,
    ) -> Result<LocalSite> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        // Check admin permissions
        if !user.has_permission(AdminPerms::Config) {
            return Err(TinyBoardsError::from_message(403, "Admin permissions required").into());
        }

        let _site = DbLocalSite::read(pool).await?;

        let form = LocalSiteForm {
            name: input.name,
            description: input.description.map(Some),
            icon: input.icon.map(Some),
            primary_color: input.primary_color.map(Some),
            secondary_color: input.secondary_color.map(Some),
            hover_color: input.hover_color.map(Some),
            welcome_message: input.welcome_message,
            default_theme: input.default_theme,
            enable_downvotes: input.enable_downvotes,
            open_registration: input.open_registration,
            enable_nsfw: input.enable_nsfw,
            board_creation_admin_only: input.board_creation_admin_only,
            require_email_verification: input.require_email_verification,
            require_application: input.require_application,
            application_question: input.application_question.map(Some),
            private_instance: input.private_instance,
            invite_only: input.invite_only,
            updated: Some(naive_now()),
            ..LocalSiteForm::default()
        };

        let updated_site = DbLocalSite::update(pool, &form)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update site configuration"))?;

        Ok(LocalSite::from(updated_site))
    }
}