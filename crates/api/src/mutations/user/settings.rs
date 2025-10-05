use crate::helpers::files::upload::{delete_file, upload_file_opendal};
use crate::{structs::user::User, DbPool, LoggedInUser, Settings};
use async_graphql::*;
use tinyboards_db::{
    models::{
        user::user::{User as DbUser, UserForm},
        site::site::Site as DbSite,
    },
    traits::Crud,
    utils::naive_now,
};
use tinyboards_utils::{parser::parse_markdown_opt, utils::custom_body_parsing,
		       passhash::{hash_password, verify_password}, TinyBoardsError};

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
        signature: Option<String>,
        show_nsfw: Option<bool>,
        default_sort_type: Option<i16>,
        default_listing_type: Option<i16>,
        avatar: Option<Upload>,
        banner: Option<Upload>,
        profile_background: Option<Upload>,
        email: Option<String>,
    ) -> Result<User> {
        let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;
        let settings = ctx.data::<Settings>()?.as_ref();

        let site = DbSite::read(pool).await?;

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
	    if name.to_lowercase() != v.name.to_lowercase() {
		return Err(TinyBoardsError::from_message(
		    403,
		    "You can only change the capitalization of your @username."
		).into());
	    }
	}

	let name = name.unwrap_or(v.name.clone());

        let display_name = match display_name {
            Some(display_name) => display_name,
            None => match v.display_name {
                Some(ref display_name) => display_name.clone(),
                None => v.name.clone(),
            },
        };

        let show_nsfw = show_nsfw.unwrap_or(v.show_nsfw);
        let default_sort_type = default_sort_type.unwrap_or(v.default_sort_type);
        let default_listing_type = default_listing_type.unwrap_or(v.default_listing_type);

        // Check bio length
        if let Some(ref bio) = bio {
            if bio.len() > 255 {
                return Err(
                    TinyBoardsError::from_message(400, "Bio too long (max 255 chars)").into(),
                );
            }
        };

        // Check signature length
        if let Some(ref sig) = signature {
            if sig.len() > 500 {
                return Err(
                    TinyBoardsError::from_message(400, "Signature too long (max 500 chars)").into(),
                );
            }
        };

        // Parse bio markdown
        let bio_html = match bio {
            Some(ref bio) => {
                let bio_html = parse_markdown_opt(bio);
                Some(custom_body_parsing(&bio_html.unwrap_or_default(), settings))
            }
            None => v.bio_html.clone(),
        };

        // Parse signature with emoji support
        let signature_parsed = match signature {
            Some(ref sig) => {
                if sig.is_empty() {
                    None
                } else {
                    // Parse custom emojis in signature if site has emoji enabled
                    let sig_with_emojis = if site.emoji_enabled {
                        use crate::utils::emoji::EmojiParser;
                        let parser = EmojiParser::new(pool, None).await?;
                        // No usage increment for signatures, just parse
                        parser.parse_emojis_to_html(sig)
                    } else {
                        sig.clone()
                    };
                    Some(sig_with_emojis)
                }
            }
            None => v.signature.clone(),
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
            None => v.email.clone(),
        };

        // If saving the new avatar/banner/bg is successful, the old one must be deleted from the file system
        let (has_new_avatar, has_new_banner, has_new_bg) = (
            avatar.is_some(),
            banner.is_some(),
            profile_background.is_some(),
        );
        let current_avatar = v.avatar.clone();
        let current_banner = v.banner.clone();
        let current_bg = v.profile_background.clone();

        let avatar = match avatar {
            Some(upload) => Some(
                upload_file_opendal(
                    upload,
                    Some(format!("user_{}_avatar", v.id)),
                    v.id,
                    Some(settings.media.max_avatar_size_mb),
                    ctx,
                )
                .await?
                .into(),
            ),
            None => v.avatar.clone(),
        };

        let banner = match banner {
            Some(upload) => Some(
                upload_file_opendal(
                    upload,
                    Some(format!("user_{}_banner", v.id)),
                    v.id,
                    Some(settings.media.max_banner_size_mb),
                    ctx,
                )
                .await?
                .into(),
            ),
            None => v.banner.clone(),
        };

        let profile_background = match profile_background {
            Some(upload) => Some(
                upload_file_opendal(
                    upload,
                    Some(format!("user_{}_bg", v.id)),
                    v.id,
                    Some(settings.media.max_banner_size_mb),
                    ctx,
                )
                .await?
                .into(),
            ),
            None => v.profile_background.clone(),
        };

        // Save changes to db
        // Save when the update happened
        let updated = Some(naive_now());

        let user_form = UserForm {
	    name: Some(name.clone()),
            bio,
            bio_html,
            signature: Some(signature_parsed.clone()),
            display_name: Some(display_name),
            avatar: Some(avatar.clone()),
            banner: Some(banner.clone()),
            profile_background: Some(profile_background.clone()),
            email: Some(email.clone()),
            show_nsfw: Some(show_nsfw),
            default_listing_type: Some(default_listing_type),
            default_sort_type: Some(default_sort_type),
            updated: Some(updated),
            ..UserForm::default()
        };

        let res = DbUser::update(pool, v.id, &user_form).await;

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

        if has_new_email {
            // TODO: send verification email
        }

	// TODO: broadcast update to fedi
	let p = DbUser::read(pool, v.id).await?;
	Ok(User::from(p))
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

	if !verify_password(v.passhash.as_str(), old_password.as_str()) {
	    return Err(TinyBoardsError::from_message(403, "Current password incorrect.").into());
	}

	let new_passhash = hash_password(new_password);

	if new_passhash.as_str() == v.passhash.as_str() {
	    return Err(TinyBoardsError::from_message(400, "Old password and new password matches.").into());
	}

	let user_form = UserForm {
	    passhash: Some(new_passhash),
	    ..UserForm::default()
	};

	DbUser::update(pool, v.id, &user_form).await?;

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

	if !verify_password(v.passhash.as_str(), password.as_str()) {
	    return Err(TinyBoardsError::from_message(403, "Password incorrect.").into());
	}

	if let Some(ref avatar) = v.avatar {
	    delete_file(pool, avatar).await?;
	}

	if let Some(ref banner) = v.banner {
	    delete_file(pool, banner).await?;
	}

	if let Some(ref bg) = v.profile_background {
	    delete_file(pool, bg).await?;
	}

	DbUser::delete(pool, v.id).await?;

        Ok(DeleteAccountResponse {
	    success: true,
	    message: String::from("Account deleted successfully. RIP.")
	})
    }
}