use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    sensitive::Sensitive,
    user::{LoginResponse, SaveUserSettings},
    utils::{blocking, get_user_view_from_jwt, require_user, send_verification_email},
    request::{purge_image_from_pictrs, upload_image_to_pictrs, PictrsUploadResponse},
};
use tinyboards_db::{
    models::site::site::Site,
    models::user::users::{User, UserForm},
    utils::{diesel_option_overwrite, naive_now},
};
use tinyboards_utils::{claims::Claims, error::TinyBoardsError};
use url::Url;

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

        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        let site = blocking(context.pool(), move |conn| Site::read_local(conn)).await??;


        let mut avatar_upload_response: Option<PictrsUploadResponse> = None;
        if let Some(avatar) = data.avatar.clone() {
            if avatar.contains("base64,") && auth.is_some() {
                avatar_upload_response = Some(upload_image_to_pictrs(context.client(), context.settings(), Some(avatar), None).await?);
            } else {
                avatar_upload_response = Some(upload_image_to_pictrs(context.client(), context.settings(), None, Some(avatar)).await?);
            }
        }
        
        let mut banner_upload_response: Option<PictrsUploadResponse> = None;
        if let Some(banner) = data.banner.clone() {
            if banner.contains("base64,") && auth.is_some() {
                banner_upload_response = Some(upload_image_to_pictrs(context.client(), context.settings(), Some(banner), None).await?);
            } else {
                banner_upload_response = Some(upload_image_to_pictrs(context.client(), context.settings(), None, Some(banner)).await?);
            }
        }

        let mut signature_upload_response: Option<PictrsUploadResponse> = None;
        if let Some(signature) = data.signature.clone() {
            if signature.contains("base64,") && auth.is_some() {
                signature_upload_response = Some(upload_image_to_pictrs(context.client(), context.settings(), Some(signature), None).await?);
            } else {
                signature_upload_response = Some(upload_image_to_pictrs(context.client(), context.settings(), None, Some(signature)).await?);
            }
        }

        let avatar = match avatar_upload_response {
            Some(resp) => diesel_option_overwrite(&Some(resp.url)),
            None => None,
        };

        let banner = match banner_upload_response {
            Some(resp) => diesel_option_overwrite(&Some(resp.url)),
            None => None,
        };

        let signature = match signature_upload_response {
            Some(resp) => diesel_option_overwrite(&Some(resp.url)),
            None => None,
        };

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

        // delete old images if new images applied
        let current_avatar = user.avatar.clone().unwrap_or_default();
        let current_banner = user.banner.clone().unwrap_or_default();
        let current_signature = user.signature.clone().unwrap_or_default();

        if let Some(Some(avatar)) = avatar.clone() {
            if avatar != current_avatar && !avatar.is_empty() && !current_avatar.is_empty() {
                purge_image_from_pictrs(context.client(), context.settings(), &Url::parse(current_avatar.as_str()).unwrap()).await?;
            }
        };

        if let Some(Some(banner)) = banner.clone() {
            if banner != current_banner && !banner.is_empty() && current_banner.is_empty() {
                purge_image_from_pictrs(context.client(), context.settings(), &Url::parse(current_banner.as_str()).unwrap()).await?;
            }
        };

        if let Some(Some(signature)) = signature.clone() {
            if signature != current_signature && !signature.is_empty() && !current_signature.is_empty() {
                purge_image_from_pictrs(context.client(), context.settings(), &Url::parse(current_signature.as_str()).unwrap()).await?;
            }
        };

        // send a new verification email if email gets changed and email verification is required
        if site.email_verification_required {
            if let Some(ref email) = email {
                let previous_email = match user.email {
                    Some(ref email) => String::from(email),
                    None => String::from(""),
                };
                if previous_email.ne(email) {
                    send_verification_email(&user, email, context.pool(), context.settings())
                        .await?;
                }
            }
        }

        if email.is_none() && site.email_verification_required {
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

        let update_form = UserForm {
            bio,
            email,
            show_nsfw: data.show_nsfw,
            theme: data.theme.clone(),
            avatar,
            signature,
            banner,
            default_listing_type,
            default_sort_type,
            updated,
            ..UserForm::default()
        };

        // perform settings update
        blocking(context.pool(), move |conn| {
            User::update_settings(conn, user.id, &update_form)
                .map_err(|_| TinyBoardsError::from_message(500, "could not update user settings"))
        })
        .await??;

        let updated_user_view =
            get_user_view_from_jwt(auth, context.pool(), context.master_key()).await?;

        let new_jwt = Claims::jwt(
            updated_user_view.user.id,
            &context.master_key().jwt,
            &context.settings().hostname,
        )?;

        // return the jwt
        Ok(LoginResponse {
            jwt: Sensitive::new(new_jwt),
            user: updated_user_view,
        })
    }
}