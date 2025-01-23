use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    download_yt_audio::save_audio_from_yt_link,
    person::{LoginResponse, SaveUserSettings},
    sensitive::Sensitive,
    utils::{purge_local_image_by_url, require_user, send_verification_email},
};
use tinyboards_db::newtypes::DbUrl;
use tinyboards_db::{
    models::person::person::PersonForm,
    models::{
        person::{local_user::*, person::Person},
        site::{local_site::LocalSite, uploads::*},
    },
    utils::naive_now,
};
use tinyboards_db_views::structs::LoggedInUserView;
use tinyboards_utils::{
    claims::Claims, error::TinyBoardsError, parser::parse_markdown_opt, utils::custom_body_parsing,
};

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
            .not_banned()
            .unwrap()?;

        let site = LocalSite::read(context.pool()).await?;

        // delete old images if new images applied
        let current_avatar = view.person.avatar.clone();
        let current_banner = view.person.banner.clone();
        let current_background = view.person.profile_background.clone();
        let current_signature = view.person.signature.clone();
        let current_profile_music = view.person.profile_music.clone();
        let current_profile_music_youtube = view.person.profile_music_youtube.clone();

        let avatar = data.avatar.clone();
        let banner = data.banner.clone();
        let profile_background = data.profile_background.clone();
        let signature = data.signature.clone();

        let bio = data.bio.clone();
        let display_name = data.display_name.clone();

        let profile_music_youtube = data.profile_music.clone();

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

        let profile_music: Option<Option<DbUrl>> = match profile_music_youtube.clone() {
            Some(yt_link) => {
                let path = match current_profile_music_youtube {
                    // user has set a profile music before
                    Some(current_profile_music_youtube) => {
                        // if it hasn't changed, leave it alone
                        if yt_link == current_profile_music_youtube.to_string() {
                            current_profile_music
                        } else {
                            // it has changed
                            let path =
                                save_audio_from_yt_link(yt_link, view.person.id, context).await?;

                            Some(path)
                        }
                    }
                    // not yet
                    None => {
                        let path =
                            save_audio_from_yt_link(yt_link, view.person.id, context).await?;

                        Some(path)
                    }
                };
                Some(path)
            }
            None => None,
        };

        if let Some(avatar) = avatar.clone() {
            if let Some(current_avatar) = current_avatar.clone() {
                if avatar != current_avatar
                    && !avatar.to_string().is_empty()
                    && !current_avatar.to_string().is_empty()
                {
                    let r = purge_local_image_by_url(context.pool(), &current_avatar).await;

                    if let Err(_) = r {
                        eprintln!("Failed to purge file: {} - ignoring, please delete manually if it's really an error", current_avatar.path());
                    }
                }
            }

            // Check if avatar is valid
            let db_avatar = Upload::find_by_url(context.pool(), &avatar)
                .await
                .map_err(|e| {
                    TinyBoardsError::from_error_message(
                        e,
                        500,
                        "Something went wrong while checking profile picture",
                    )
                })?;

            // Ownership check
            if let Some(default_avatar) = site.default_avatar {
                if default_avatar != avatar.to_string() && db_avatar.person_id != view.person.id {
                    return Err(TinyBoardsError::from_message(
                        400,
                        "That image is NOT yours.",
                    ));
                }
            } else if db_avatar.person_id != view.person.id {
                return Err(TinyBoardsError::from_message(
                    400,
                    "That image is NOT yours.",
                ));
            }

            // Size check
            if db_avatar.size > 2 * 1024 * 1024 {
                return Err(TinyBoardsError::from_message(
                    403,
                    "Max size for avatars is 2MB. Please.",
                ));
            }
        };

        if let Some(banner) = banner.clone() {
            if let Some(current_banner) = current_banner {
                if banner != current_banner
                    && !banner.to_string().is_empty()
                    && !current_banner.to_string().is_empty()
                {
                    let r = purge_local_image_by_url(context.pool(), &current_banner).await;

                    if let Err(_) = r {
                        eprintln!("Failed to purge file: {} - ignoring, please delete manually if it's really an error", current_banner.path());
                    }
                }
            }

            // Check if banner is valid
            let db_banner = Upload::find_by_url(context.pool(), &banner)
                .await
                .map_err(|e| {
                    TinyBoardsError::from_error_message(
                        e,
                        500,
                        "Something went wrong while checking banner",
                    )
                })?;

            // Ownership check
            if db_banner.person_id != view.person.id {
                return Err(TinyBoardsError::from_message(
                    400,
                    "That image is NOT yours.",
                ));
            }

            // Size check
            if db_banner.size > 5 * 1024 * 1024 {
                return Err(TinyBoardsError::from_message(
                    403,
                    "Stop trying to get past the 5MB size limit, nerd.",
                ));
            }
        };

        if let Some(profile_background) = profile_background.clone() {
            if let Some(current_background) = current_background {
                if profile_background != current_background
                    && !profile_background.to_string().is_empty()
                    && !current_background.to_string().is_empty()
                {
                    let r = purge_local_image_by_url(context.pool(), &current_background).await;

                    if let Err(_) = r {
                        eprintln!("Failed to purge file: {} - ignoring, please delete manually if it's really an error", current_background.path());
                    }
                }
            }

            // Check if banner is valid
            let db_background = Upload::find_by_url(context.pool(), &profile_background)
                .await
                .map_err(|e| {
                    TinyBoardsError::from_error_message(
                        e,
                        500,
                        "Something went wrong while checking background",
                    )
                })?;

            // Ownership check
            if db_background.person_id != view.person.id {
                return Err(TinyBoardsError::from_message(
                    400,
                    "That image is NOT yours.",
                ));
            }

            // Size check
            if db_background.size > 5 * 1024 * 1024 {
                return Err(TinyBoardsError::from_message(
                    403,
                    "Size limit for profile background is 5 (FIVE!!!) megabytes bruv",
                ));
            }
        };

        if let Some(signature) = signature.clone() {
            if let Some(current_signature) = current_signature.clone() {
                if signature != current_signature
                    && !signature.to_string().is_empty()
                    && !current_signature.to_string().is_empty()
                {
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
                    send_verification_email(
                        &view.local_user,
                        email,
                        context.pool(),
                        context.settings(),
                    )
                    .await?;
                }
            }
        }

        if email.is_none() && site.require_email_verification {
            return Err(TinyBoardsError::from_message(400, "email required"));
        }

        if let Some(bio) = &bio {
            // seems sort of arbitrary? do we want a setting for this length somewhere?
            if bio.chars().count() > 255 {
                return Err(TinyBoardsError::from_message(400, "bio too long"));
            }
        }

        let bio_html = if let Some(ref bio) = bio {
            let bio_html = parse_markdown_opt(bio);
            Some(custom_body_parsing(
                &bio_html.unwrap_or_default(),
                context.settings(),
            ))
        } else {
            view.person.bio_html
        };

        if let Some(display_name) = &display_name {
            if display_name.chars().count() < 2 || display_name.chars().count() > 30 {
                return Err(TinyBoardsError::from_message(
                    400,
                    "display name must be between 2 and 30 characters long",
                ));
            }
        }

        let default_listing_type = data.default_listing_type;
        let default_sort_type = data.default_sort_type;
        // grabbing the current timestamp for the update
        let updated = Some(naive_now());

        let person_form = PersonForm {
            bio,
            bio_html,
            display_name,
            avatar: avatar.clone(),
            signature: signature.clone(),
            banner: banner.clone(),
            profile_background: Some(profile_background.clone()),
            profile_music,
            profile_music_youtube: Some(profile_music_youtube),
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
            .map_err(|_| {
                TinyBoardsError::from_message(500, "could not update local user settings")
            })?;

        // perform settings update for person
        Person::update_settings(context.pool(), view.person.id, &person_form)
            .await
            .map_err(|_| TinyBoardsError::from_message(500, "could not update person settings"))?;

        let updated_user_view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        let new_jwt = Claims::jwt(
            updated_user_view.local_user.id,
            &context.master_key().jwt,
            &context.settings().hostname,
        )?;

        // get the LoggedInUserView
        let logged_in_view = LoggedInUserView::read(
            context.pool(),
            updated_user_view.person.id,
            updated_user_view.local_user.admin_level,
        )
        .await?;

        // return the jwt
        Ok(LoginResponse {
            jwt: Sensitive::new(new_jwt),
            user: logged_in_view,
        })
    }
}
