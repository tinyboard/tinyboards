use crate::helpers::files::upload::{delete_file, upload_file};
use crate::{structs::person::Person, DbPool, LoggedInUser, Settings};
use async_graphql::*;
use tinyboards_db::{
    models::{
        person::{
            local_user::{LocalUser as DbLocalUser, LocalUserForm},
            person::{Person as DbPerson, PersonForm},
	    user::User as DbUser
        },
        site::local_site::LocalSite as DbLocalSite,
    },
    utils::naive_now,
};
use tinyboards_utils::{parser::parse_markdown_opt, utils::custom_body_parsing,
		       passhash::{hash_password, verify_password}, TinyBoardsError};

//use tinyboards_db::models::site::uploads::Upload as DbUpload;

#[derive(Default)]
pub struct UpdateSettings;

#[derive(SimpleObject)]
pub struct ChangePwResponse {
    pub success: bool,
}

#[derive(SimpleObject)]
pub struct DeleteAccountResponse {
    pub success: bool,
    pub message: String,
}

#[Object]
impl UpdateSettings {
    pub async fn update_settings(
        &self,
        ctx: &Context<'_>,
	name: Option<String>,
        display_name: Option<String>,
        bio: Option<String>,
        show_nsfw: Option<bool>,
        default_sort_type: Option<i16>,
        default_listing_type: Option<i16>,
        avatar: Option<Upload>,
        banner: Option<Upload>,
        profile_background: Option<Upload>,
        email: Option<String>,
    ) -> Result<Person> {
        let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;
        let settings = ctx.data::<Settings>()?.as_ref();
        let local_user = v.local_user.as_ref().unwrap();

        let site = DbLocalSite::read(pool).await?;

        if let Some(ref display_name) = display_name {
            let l = display_name.len();
            if l < 1 || l > 30 {
                return Err(TinyBoardsError::from_message(
                    400,
                    "Display name must be between 1 and 30 characters long.",
                )
                .into());
            }
        }

	// only the capitalization of the @username can be changed
	if let Some(ref name) = name {
	    if name.to_lowercase() != v.person.name.to_lowercase() {
		return Err(TinyBoardsError::from_message(
		    403,
		    "You can only change the capitalization of your @username."
		).into());
	    }
	}

	let name = name.unwrap_or(v.person.name.clone());

        let display_name = match display_name {
            Some(display_name) => display_name,
            None => match v.person.display_name {
                Some(ref display_name) => display_name.clone(),
                None => v.person.name.clone(),
            },
        };

        let show_nsfw = show_nsfw.unwrap_or(local_user.show_nsfw);
        let default_sort_type = default_sort_type.unwrap_or(local_user.default_sort_type);
        let default_listing_type = default_listing_type.unwrap_or(local_user.default_listing_type);

        // Check bio length
        if let Some(ref bio) = bio {
            if bio.len() > 255 {
                return Err(
                    TinyBoardsError::from_message(400, "Bio too long (max 255 chars)").into(),
                );
            }
        };

        // Parse bio markdown
        let bio_html = match bio {
            Some(ref bio) => {
                let bio_html = parse_markdown_opt(bio);
                Some(custom_body_parsing(&bio_html.unwrap_or_default(), settings))
            }
            None => v.person.bio_html.clone(),
        };

        let has_new_email = email.is_some();
        let email_verification_required = site.require_email_verification;

        if let Some(ref email) = email {
            if email.is_empty() && email_verification_required {
                return Err(TinyBoardsError::from_message(
                    403,
                    "A verified email address is required. You cannot remove your email address.",
                )
                .into());
            }
        }

        let email = match email {
            Some(email) => Some(email),
            None => local_user.email.clone(),
        };

        // If saving the new avatar/banner/bg is successful, the old one must be deleted from the file system
        let (has_new_avatar, has_new_banner, has_new_bg) = (
            avatar.is_some(),
            banner.is_some(),
            profile_background.is_some(),
        );
        let current_avatar = v.person.avatar.clone();
        let current_banner = v.person.banner.clone();
        let current_bg = v.person.profile_background.clone();

        let avatar = match avatar {
            Some(upload) => Some(
                upload_file(
                    upload,
                    Some(format!("person_{}_avatar", v.person.id)),
                    v.person.id,
                    Some(2),
                    ctx,
                )
                .await?
                .into(),
            ),
            None => v.person.avatar.clone(),
        };

        let banner = match banner {
            Some(upload) => Some(
                upload_file(
                    upload,
                    Some(format!("person_{}_banner", v.person.id)),
                    v.person.id,
                    Some(5),
                    ctx,
                )
                .await?
                .into(),
            ),
            None => v.person.banner.clone(),
        };

        let profile_background = match profile_background {
            Some(upload) => Some(
                upload_file(
                    upload,
                    Some(format!("person_{}_bg", v.person.id)),
                    v.person.id,
                    Some(5),
                    ctx,
                )
                .await?
                .into(),
            ),
            None => v.person.profile_background.clone(),
        };

        // Save changes to db
        // Save when the update happened
        let updated = Some(naive_now());

        let person_form = PersonForm {
	    name: Some(name.clone()),
            bio,
            bio_html,
            display_name: Some(display_name),
            avatar: avatar.clone(),
            banner: banner.clone(),
            profile_background: Some(profile_background.clone()),
            updated,
            ..PersonForm::default()
        };

        let res = DbPerson::update_settings(pool, v.person.id, &person_form).await;

        // Settings failed to save - revert any changes to pfp, banner and bg
        // (new uploads will not be used and thus must be deleted so they don't occupy space)
        if let Err(e) = res {
            if has_new_avatar {
                // Safe: it is known that the user has uploaded a new avatar
                let avatar = avatar.unwrap();
                if let Err(_) = delete_file(pool, &avatar).await {
                    eprintln!("ERROR: Failed to delete redundant file {}", avatar);
                }
            }

            if has_new_banner {
                let banner = banner.unwrap();
                if let Err(_) = delete_file(pool, &banner).await {
                    eprintln!("ERROR: Failed to delete redundant file {}", banner);
                }
            }

            if has_new_bg {
                let bg = profile_background.unwrap();
                if let Err(_) = delete_file(pool, &bg).await {
                    eprintln!("ERROR: Failed to delete redundant file {}", bg);
                }
            }

            return Err(TinyBoardsError::from_error_message(
                e,
                500,
                "Saving settings failed due to an internal server error",
            )
            .into());
        } else {
            // Saving settings successful: old files should be deleted
            if has_new_avatar {
                if let Some(old_avatar) = current_avatar {
                    if let Err(_) = delete_file(pool, &old_avatar).await {
                        eprintln!("ERROR: Failed to delete redundant file {}", old_avatar);
                    }
                }
            }

            if has_new_banner {
                if let Some(old_banner) = current_banner {
                    if let Err(_) = delete_file(pool, &old_banner).await {
                        eprintln!("ERROR: Failed to delete redundant file {}", old_banner);
                    }
                }
            }

            if has_new_bg {
                if let Some(old_bg) = current_bg {
                    if let Err(_) = delete_file(pool, &old_bg).await {
                        eprintln!("ERROR: Failed to delete redundant file {}", old_bg);
                    }
                }
            }
        }

        let local_user_form = LocalUserForm {
	    name: Some(name),
            email: Some(email),
            show_nsfw: Some(show_nsfw),
            default_listing_type: Some(default_listing_type),
            default_sort_type: Some(default_sort_type),
            updated: Some(updated.clone()),
            ..LocalUserForm::default()
        };

        DbLocalUser::update_settings(pool, local_user.id, &local_user_form)
            .await
            .map_err(|_| TinyBoardsError::from_message(500, "Failed to save settings due to an internal server error."))?;

        if has_new_email {
            // TODO: send verification email
        }

	// TODO: broadcast update to fedi
	let p = DbPerson::get_user_by_id(pool, v.person.id).await?;
	Ok(Person::from(p))
    }

    pub async fn update_password(
        &self,
        ctx: &Context<'_>,
        old_password: String,
        new_password: String
    ) -> Result<ChangePwResponse> {
	let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user()?;
        let pool = ctx.data::<DbPool>()?;
	let local_user = v.local_user.as_ref().unwrap();

	if !verify_password(local_user.passhash.as_str(), old_password.as_str()) {
	    return Err(TinyBoardsError::from_message(403, "Current password incorrect.").into());
	}

	let new_passhash = hash_password(new_password);

	if new_passhash.as_str() == local_user.passhash.as_str() {
	    return Err(TinyBoardsError::from_message(400, "Old password and new password matches.").into());
	}

	let local_user_form = LocalUserForm {
	    passhash: Some(new_passhash),
	    ..LocalUserForm::default()
	};

	DbLocalUser::update_settings(pool, local_user.id, &local_user_form).await?;

        Ok(ChangePwResponse { success: true })
    }

    pub async fn delete_account(
        &self,
        ctx: &Context<'_>,
        password: String,
    ) -> Result<DeleteAccountResponse> {
	let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user()?;
        let pool = ctx.data::<DbPool>()?;
	let local_user = v.local_user.as_ref().unwrap();

	if !verify_password(local_user.passhash.as_str(), password.as_str()) {
	    return Err(TinyBoardsError::from_message(403, "Password incorrect.").into());
	}

	if let Some(ref avatar) = v.person.avatar {
	    delete_file(pool, avatar).await?;
	}

	if let Some(ref banner) = v.person.banner {
	    delete_file(pool, banner).await?;
	}

	if let Some(ref bg) = v.person.profile_background {
	    delete_file(pool, bg).await?;
	}

	DbPerson::delete_account(pool, v.person.id).await?;
	
        Ok(DeleteAccountResponse {
	    success: true,
	    message: String::from("Account deleted successfully. RIP.")
	})
    }
}
