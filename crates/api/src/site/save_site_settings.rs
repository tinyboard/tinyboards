use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    site::{SaveSiteSettings, GetSiteSettingsResponse},
    utils::{require_user, blocking,},
    data::TinyBoardsContext,
};
use tinyboards_db::{
    models::site::site::{Site, SiteForm},
    utils::{
        naive_now,
    }, traits::Crud,
};
use tinyboards_utils::{
    error::TinyBoardsError,
};

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for SaveSiteSettings {
    type Response = GetSiteSettingsResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<GetSiteSettingsResponse, TinyBoardsError> {
        let data: &SaveSiteSettings = &self;

        // only an admin should be able to change site settings (TODO - make this owner only)
        require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        let site =
            blocking(context.pool(), move |conn| {
                Site::read_local(conn)
            })
            .await??;
        
        let name = data.name.clone();
        let description = data.description.clone();
        let enable_downvotes = data.enable_downvotes;
        let open_registration = data.open_registration;
        let enable_nsfw = data.enable_nsfw;
        let require_application = data.require_application;
        let application_question = data.application_question.clone();
        let private_instance = data.private_instance;
        let email_verification_required = data.email_verification_required;
        let invite_only = data.invite_only;
            
        if let Some(description) = &description {
            if description.chars().count() > 500 {
                return Err(TinyBoardsError::from_message("description too long"));
            }
        }

        if let Some(application_question) = &application_question {
            if application_question.chars().count() > 300 {
                return Err(TinyBoardsError::from_message("question too long"));
            }
        }

        // ensure that only one mode is active at a time
        if open_registration.is_some() 
        && require_application.is_some() 
        && invite_only.is_some() {

            let open_r = open_registration.unwrap();
            let require_a = require_application.unwrap();
            let invite_o = invite_only.unwrap();

            if require_a && invite_o && !open_r {
                return Err(TinyBoardsError::from_message("site cannot be invite only mode and application mode at the same time"));
            }

            if open_r && require_a {
                return Err(TinyBoardsError::from_message("site cannot be open registration and application mode at the same time"));
            }

            if open_r && invite_o {
                return Err(TinyBoardsError::from_message("site cannot be open registration and invite only mode at thee same time"));
            }

            if require_a && invite_o && open_r {
                return Err(TinyBoardsError::from_message("site cannot be open registration and invite only mode and application mode at the same time"));
            } 
        }

        let form = SiteForm {
            name,
            description,
            enable_downvotes,
            open_registration,
            enable_nsfw,
            require_application,
            application_question: Some(application_question),
            private_instance,
            email_verification_required,
            invite_only,
            updated: Some(Some(naive_now())),
            ..SiteForm::default()
        };

        // perform settings update
        let updated_site = 
            blocking(context.pool(), move |conn| {
                Site::update(conn, site.id, &form)
            })
            .await??;
            
        Ok(GetSiteSettingsResponse {
            name: updated_site.name,
            description: updated_site.description.unwrap_or_default(),
            enable_downvotes: updated_site.enable_downvotes,
            open_registration: updated_site.open_registration,
            enable_nsfw: updated_site.enable_nsfw,
            require_application: updated_site.require_application,
            application_question: updated_site.application_question.unwrap_or_default(),
            private_instance: updated_site.private_instance,
            email_verification_required: updated_site.email_verification_required,
            invite_only: updated_site.invite_only,            
        })
    }
}