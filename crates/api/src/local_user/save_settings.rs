use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    sensitive::Sensitive,
    user::{LoginResponse, SaveUserSettings},
    utils::{get_local_user_view_from_jwt, require_user, send_verification_email, purge_local_image_by_url},
};
use tinyboards_db::{
    models::{person::person::PersonForm},
    models::{person::{local_user::*, person::Person}, apub::local_site::LocalSite},
    utils::{diesel_option_overwrite, naive_now},
};
use tinyboards_utils::{claims::Claims, error::TinyBoardsError};

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for SaveUserSettings {
    type Response = LoginResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<LoginResponse, TinyBoardsError> {
        let data: &SaveUserSettings = &self;

        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        let site = LocalSite::read_local(context.pool()).await?;
        
        // delete old images if new images applied
        let current_avatar = view.person.avatar.clone().unwrap_or_default();
        let current_banner = view.person.banner.clone().unwrap_or_default();
        let current_signature = view.person.signature.clone().unwrap_or_default();
        
        let avatar = data.avatar.clone();
        let banner = data.banner.clone();
        let signature = data.signature.clone();

        let bio = diesel_option_overwrite(&data.bio);
        let email_deref = data.email.as_deref().map(str::to_lowercase);
        let email = match email_deref {
            Some(email) => {
                if email.is_empty() {
                    None
                } else {
                    Some(email)
                }
            }
            None => None,
        };

        if let Some(avatar) = avatar.clone() {
            if avatar != current_avatar && !avatar.is_empty() && !current_avatar.is_empty() {
                purge_local_image_by_url(context.pool(), &current_avatar).await?;
            }
        };

        if let Some(banner) = banner.clone() {
            if banner != current_banner && !banner.is_empty() && !current_banner.is_empty() {
                purge_local_image_by_url(context.pool(), &current_banner).await?;
            }
        };

        if let Some(signature) = signature.clone() {
            if signature != current_signature && !signature.is_empty() && !current_signature.is_empty() {
                purge_local_image_by_url(context.pool(), &current_signature).await?;
            }
        };

        // send a new verification email if email gets changed and email verification is required
        if site.require_email_verification {
            if let Some(ref email) = email {
                let previous_email = match view.local_user.email {
                    Some(ref email) => String::from(email),
                    None => String::from(""),
                };
                if previous_email.ne(email) {
                    send_verification_email(&view.local_user, email, context.pool(), context.settings())
                        .await?;
                }
            }
        }

        if email.is_none() && site.require_email_verification {
            return Err(TinyBoardsError::from_message(400, "email required"));
        }

        if let Some(Some(bio)) = &bio {
            // seems sort of arbitrary? do we want a setting for this length somewhere?
            if bio.chars().count() > 300 {
                return Err(TinyBoardsError::from_message(400, "bio too long"));
            }
        }

        let default_listing_type = data.default_listing_type;
        let default_sort_type = data.default_sort_type;
        // grabbing the current timestamp for the update
        let updated = Some(naive_now());

        let person_form = PersonForm {
            bio,
            avatar: Some(avatar.clone()),
            signature: Some(signature.clone()),
            banner: Some(banner.clone()),
            updated: updated,
            ..PersonForm::default()
        };

        let local_user_form = LocalUserForm {
            email: Some(email),
            show_nsfw: data.show_nsfw,
            theme: data.theme.clone(),
            default_listing_type,
            default_sort_type,
            updated: Some(updated.clone()),
            ..LocalUserForm::default()
        };

        // perform settings update for local_user
        LocalUser::update_settings(context.pool(), view.local_user.id, &local_user_form)
            .await
            .map_err(|_| TinyBoardsError::from_message(500, "could not update local user settings"))?;

        // perform settings update for person
        Person::update_settings(context.pool(), view.person.id, &person_form)
            .await
            .map_err(|_| TinyBoardsError::from_message(500, "could not update person settings"))?;


        let updated_user_view =
            get_local_user_view_from_jwt(auth, context.pool(), context.master_key()).await?;

        let new_jwt = Claims::jwt(
            updated_user_view.local_user.id,
            &context.master_key().jwt,
            &context.settings().hostname,
        )?;
        
        // TODO: look into this response, are we really wanting to return the entire local user view (has all fields from Person and LocalUser)

        // return the jwt
        Ok(LoginResponse {
            jwt: Sensitive::new(new_jwt),
            user: updated_user_view,
        })
    }
}