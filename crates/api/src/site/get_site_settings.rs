use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{GetSiteSettings, GetSiteSettingsResponse},
    utils::{
        blocking,
    }
};
use tinyboards_db::{
    models::site::site::Site, SiteMode
};
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

        let site =
            blocking(context.pool(), move |conn| {
                Site::read_local(conn)
            })
            .await??;
        
        let mut site_mode = SiteMode::OpenMode;

        if site.require_application {
            site_mode = SiteMode::ApplicationMode;
        }

        if site.invite_only {
            site_mode = SiteMode::InviteMode;
        }

            
        Ok(GetSiteSettingsResponse {
            name: site.name,
            description: site.description.unwrap_or_default(),
            site_mode,
            enable_downvotes: site.enable_downvotes,
            enable_nsfw: site.enable_nsfw,
            application_question: site.application_question.unwrap_or_default(),
            private_instance: site.private_instance,
            email_verification_required: site.email_verification_required,            
        })
    }
}