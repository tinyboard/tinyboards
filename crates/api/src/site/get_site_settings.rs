use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{GetSiteSettings, GetSiteSettingsResponse},
};
use tinyboards_db::{models::site::local_site::LocalSite, SiteMode};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetSiteSettings {
    type Response = GetSiteSettingsResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        _: Option<&str>,
    ) -> Result<GetSiteSettingsResponse, TinyBoardsError> {
        let site = LocalSite::read(context.pool()).await?;

        let mut site_mode = SiteMode::OpenMode;

        if site.require_application {
            site_mode = SiteMode::ApplicationMode;
        }

        if site.invite_only {
            site_mode = SiteMode::InviteMode;
        }

        Ok(GetSiteSettingsResponse {
            site_mode,
            name: site.name,
            primary_color: site.primary_color,
            secondary_color: site.secondary_color,
            hover_color: site.hover_color,
            description: site.description,
            icon: site.icon,
            enable_downvotes: site.enable_downvotes,
            enable_nsfw: site.enable_nsfw,
            application_question: site.application_question.unwrap_or_default(),
            private_instance: site.private_instance,
            require_email_verification: site.require_email_verification,
            default_avatar: site.default_avatar.unwrap_or_default(),
            welcome_message: site.welcome_message,
            boards_enabled: site.boards_enabled,
            board_creation_admin_only: site.board_creation_admin_only,
        })
    }
}
