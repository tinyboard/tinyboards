use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{GetSiteSettingsResponse, SaveSiteSettings},
    utils::{get_current_site_mode, purge_local_image_by_url, require_user},
};
use tinyboards_db::{
    models::{
        person::{
            local_user::{AdminPerms, LocalUser},
            person::Person,
        },
        site::local_site::{LocalSite, LocalSiteForm},
    },
    utils::naive_now,
    SiteMode,
};
use tinyboards_utils::error::TinyBoardsError;
use url::Url;

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
            .require_admin(AdminPerms::Config)
            .unwrap()?;

        let site = LocalSite::read(context.pool()).await?;

        let current_require_app = site.require_application;
        let current_icon = site.icon.clone();
        let _current_name = site.name.clone();
        let current_default_avatar = site.default_avatar.clone();

        let new_name = data.name.clone();
        let primary_color = data.primary_color.clone();
        let secondary_color = data.secondary_color.clone();
        let hover_color = data.hover_color.clone();
        let description = data.description.clone();
        let icon = data.icon.clone();
        let site_mode = data.site_mode;
        let enable_downvotes = data.enable_downvotes;
        let enable_nsfw = data.enable_nsfw;
        let application_question = data.application_question.clone();
        let private_instance = data.private_instance;
        let email_verification_required = data.require_email_verification;
        let default_avatar = data.default_avatar.clone();
        let welcome_message = data.welcome_message.clone();

        /*if let Some(ref new_name) = new_name {
            if new_name.to_lowercase() != current_name.to_lowercase() {
                return Err(TinyBoardsError::from_message(400, "You can only change the capitalization of your site's name!"));
            }
        }

        if let Some(ref new_color) = new_color {
            if new_color.len() > 12 {
                return Err(TinyBoardsError::from_message(400, "Color must be a valid RGB value"));
            }
        }*/

        if let Some(ref icon) = icon {
            if let Some(ref current_icon) = current_icon {
                if icon != current_icon && !icon.is_empty() && !current_icon.is_empty() {
                    let r = purge_local_image_by_url(
                        context.pool(),
                        &Url::parse(current_icon).unwrap().into(),
                    )
                    .await;

                    if let Err(_) = r {
                        eprintln!("Failed to purge file: {} - ignoring, please delete manually if it's really an error", current_icon);
                    }
                }
            }
        }

        if let Some(application_question) = &application_question {
            if application_question.chars().count() > 300 {
                return Err(TinyBoardsError::from_message(400, "question too long"));
            }
        }

        if let Some(welcome_message) = &welcome_message {
            if welcome_message.chars().count() > 255 {
                return Err(TinyBoardsError::from_message(
                    400,
                    "welcome message too long",
                ));
            }
        }

        // update the default avatar url if it is provided and different then the current one set in local_site
        if current_default_avatar.is_none() && default_avatar.is_some() {
            let old_avatar_url = format!(
                "{}/media/default_pfp.png",
                context.settings().get_protocol_and_hostname()
            );
            let new_avatar_url = default_avatar.clone().unwrap();
            Person::update_default_avatar(context.pool(), old_avatar_url, new_avatar_url).await?;
        }

        if current_default_avatar.is_some() && default_avatar.is_some() {
            let old_avatar_url = current_default_avatar.unwrap();
            let new_avatar_url = default_avatar.clone().unwrap();
            Person::update_default_avatar(context.pool(), old_avatar_url, new_avatar_url).await?;
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
            primary_color: Some(primary_color),
            secondary_color: Some(secondary_color),
            hover_color: Some(hover_color),
            description: Some(description),
            icon: Some(icon),
            enable_downvotes,
            open_registration,
            enable_nsfw,
            require_application,
            application_question: Some(application_question),
            private_instance,
            require_email_verification: email_verification_required,
            invite_only,
            default_avatar: Some(default_avatar),
            welcome_message,
            updated: Some(naive_now()),
            ..LocalSiteForm::default()
        };

        // perform settings update
        let updated_local_site = LocalSite::update(context.pool(), &form).await?;

        Ok(GetSiteSettingsResponse {
            name: updated_local_site.name,
            primary_color: updated_local_site.primary_color,
            secondary_color: updated_local_site.secondary_color,
            hover_color: updated_local_site.hover_color,
            description: updated_local_site.description,
            icon: updated_local_site.icon,
            site_mode: get_current_site_mode(&site, &site_mode),
            enable_downvotes: updated_local_site.enable_downvotes,
            enable_nsfw: updated_local_site.enable_nsfw,
            application_question: updated_local_site.application_question.unwrap_or_default(),
            private_instance: updated_local_site.private_instance,
            require_email_verification: updated_local_site.require_email_verification,
            default_avatar: updated_local_site.default_avatar.unwrap_or_default(),
            welcome_message: updated_local_site.welcome_message,
        })
    }
}
