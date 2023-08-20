use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    sensitive::Sensitive,
    person::{LoginResponse, SaveUserSettings},
    utils::{get_local_user_view_from_jwt, require_user, send_verification_email, purge_local_image_by_url},
};
use tinyboards_db::{
    models::{person::person::PersonForm},
    models::{person::{local_user::*, person::Person}, site::local_site::LocalSite},
    utils::{naive_now},
};
use tinyboards_db_views::structs::LoggedInUserView;
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

        let site = LocalSite::read(context.pool()).await?;
        
        // delete old images if new images applied
        let current_avatar = view.person.avatar.clone();
        let current_banner = view.person.banner.clone();
        let current_signature = view.person.signature.clone();
        
        let avatar = data.avatar.clone();
        let banner = data.banner.clone();
        let signature = data.signature.clone();

        let bio = data.bio.clone();
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
            if let Some(current_avatar) = current_avatar.clone() {
                if avatar != current_avatar && !avatar.to_string().is_empty() && !current_avatar.to_string().is_empty() {
                    let r = purge_local_image_by_url(context.pool(), &current_avatar).await;

                    if let Err(_) = r {
                        eprintln!("Failed to purge file: {:#?} - ignoring, please delete manually if it's really an error", current_avatar);
                    }
                }
            }
        };

        if let Some(banner) = banner.clone() {
            if let Some(current_banner) = current_banner {
                if banner != current_banner && !banner.to_string().is_empty() && !current_banner.to_string().is_empty() {
                    let r = purge_local_image_by_url(context.pool(), &current_banner).await;

                    if let Err(_) = r {
                        eprintln!("Failed to purge file: {:#?} - ignoring, please delete manually if it's really an error", current_banner);
                    }
                }
            }
        };

        if let Some(signature) = signature.clone() {
            if let Some(current_signature) = current_signature.clone() {
                if signature != current_signature && !signature.to_string().is_empty() && !current_signature.to_string().is_empty() {
                    let r = purge_local_image_by_url(context.pool(), &current_signature).await;

                    if let Err(_) = r {
                        eprintln!("Failed to purge file: {} - ignoring, please delete manually if it's really an error", current_signature);
                    }
                }            
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

        if let Some(bio) = &bio {
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
            avatar: avatar.clone(),
            signature: signature.clone(),
            banner: banner.clone(),
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
        
        // get the LoggedInUserView
        let logged_in_view = LoggedInUserView::read(context.pool(), updated_user_view.person.id).await?;

        // return the jwt
        Ok(LoginResponse {
            jwt: Sensitive::new(new_jwt),
            user: logged_in_view,
        })
    }
}