use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{GetSiteSettings, GetSiteSettingsResponse},
    utils::{
        blocking,
        require_user,
    }
};
use tinyboards_db::{
    models::site::site::Site
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetSiteSettings {
    type Response = GetSiteSettingsResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<GetSiteSettingsResponse, TinyBoardsError> {

        // only an admin should be able to view site settings
        require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        let site =
            blocking(context.pool(), move |conn| {
                Site::read_local(conn)
            })
            .await??;
            
        Ok(GetSiteSettingsResponse {
            name: site.name,
            description: site.description.unwrap_or_default(),
            enable_downvotes: site.enable_downvotes,
            open_registration: site.open_registration,
            enable_nsfw: site.enable_nsfw,
            require_application: site.require_application,
            application_question: site.application_question.unwrap_or_default(),
            private_instance: site.private_instance,
            email_verification_required: site.email_verification_required,
            invite_only: site.invite_only,            
        })
    }
}