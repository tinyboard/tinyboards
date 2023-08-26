use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{GetSiteSettingsResponse, SaveSiteSettings},
    utils::{get_current_site_mode, require_user},
};
use tinyboards_db::{
    models::{person::local_user::LocalUser, site::local_site::{LocalSite, LocalSiteForm}},
    utils::naive_now,
    SiteMode,
};
use tinyboards_utils::error::TinyBoardsError;

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

        let site = LocalSite::read(context.pool()).await?;

        let current_require_app = site.require_application;
        let current_name = site.name.clone();

        let new_name = data.name.clone();
        let new_color = data.color.clone();
        let site_mode = data.site_mode;
        let enable_downvotes = data.enable_downvotes;
        let enable_nsfw = data.enable_nsfw;
        let application_question = data.application_question.clone();
        let private_instance = data.private_instance;
        let email_verification_required = data.require_email_verification;
        let default_avatar = data.default_avatar.clone();

        if let Some(ref new_name) = new_name {
            if new_name.to_lowercase() != current_name.to_lowercase() {
                return Err(TinyBoardsError::from_message(400, "You can only change the capitalization of your site's name!"));
            }
        }

        if let Some(ref new_color) = new_color {
            if new_color.len() > 12 {
                return Err(TinyBoardsError::from_message(400, "Color must be a valid RGB value"));
            }
        }

        if let Some(application_question) = &application_question {
            if application_question.chars().count() > 300 {
                return Err(TinyBoardsError::from_message(400, "question too long"));
            }
        }

        let open_registration = match site_mode {
            Some(SiteMode::OpenMode) => Some(true),
            Some(SiteMode::ApplicationMode) => Some(false),
            Some(SiteMode::InviteMode) => Some(false),
            None => Some(site.open_registration),
        };

        let require_application = match site_mode {
            Some(SiteMode::OpenMode) => Some(false),
            Some(SiteMode::ApplicationMode) => Some(true),
            Some(SiteMode::InviteMode) => Some(false),
            None => Some(site.require_application),
        };

        // we need to toggle all unaccepted users to accepted after toggling app mode on/off
        if let Some(require_application) = require_application {
            if require_application != current_require_app {
                LocalUser::accept_all_applications(context.pool()).await?;
            }
        }

        let invite_only = match site_mode {
            Some(SiteMode::OpenMode) => Some(false),
            Some(SiteMode::ApplicationMode) => Some(false),
            Some(SiteMode::InviteMode) => Some(true),
            None => Some(site.invite_only),
        };

        let form = LocalSiteForm {
            name: new_name,
            color: Some(new_color),
            enable_downvotes,
            open_registration,
            enable_nsfw,
            require_application,
            application_question: Some(application_question),
            private_instance,
            require_email_verification: email_verification_required,
            invite_only,
            default_avatar: Some(default_avatar),
            updated: Some(naive_now()),
            ..LocalSiteForm::default()
        };

        // perform settings update
        let updated_local_site = LocalSite::update(context.pool(), &form).await?;

        Ok(GetSiteSettingsResponse {
            name: updated_local_site.name,
            color: updated_local_site.color,
            site_mode: get_current_site_mode(&site, &site_mode),
            enable_downvotes: updated_local_site.enable_downvotes,
            enable_nsfw: updated_local_site.enable_nsfw,
            application_question: updated_local_site.application_question.unwrap_or_default(),
            private_instance: updated_local_site.private_instance,
            require_email_verification: updated_local_site.require_email_verification,
            default_avatar: updated_local_site.default_avatar.unwrap_or_default(),
        })
    }
}
