/**
 * Additional Profile Management Operations
 */
use crate::{DbPool, LoggedInUser};
use async_graphql::*;
use tinyboards_db::models::user::user::{User as DbUser, UserForm};
use tinyboards_db::traits::Crud;
use tinyboards_utils::{TinyBoardsError, passhash::{hash_password, verify_password}};
use crate::utils::auth::password_length_check;
use crate::helpers::files::upload::upload_file;

#[derive(Default)]
pub struct ProfileManagement;

#[derive(InputObject)]
pub struct ChangePasswordInput {
    pub current_password: String,
    pub new_password: String,
}

#[derive(InputObject)]
pub struct UpdateUserProfileInput {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub profile_background: Option<String>,
    pub signature: Option<String>,
    pub profile_music_youtube: Option<String>,
}

#[Object]
impl ProfileManagement {
    /// Change user password with current password verification
    pub async fn change_password(
        &self,
        ctx: &Context<'_>,
        input: ChangePasswordInput,
    ) -> Result<bool> {
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        // Verify current password
        if !verify_password(&user.passhash, &input.current_password) {
            return Err(TinyBoardsError::from_message(400, "Current password is incorrect").into());
        }

        // Validate new password length
        password_length_check(&input.new_password)?;

        // Hash the new password
        let new_passhash = hash_password(input.new_password);

        let user_form = UserForm {
            passhash: Some(new_passhash),
            ..UserForm::default()
        };

        DbUser::update(pool, user.id, &user_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update password"))?;

        Ok(true)
    }

    /// Update user profile information
    pub async fn update_user_profile(
        &self,
        ctx: &Context<'_>,
        input: UpdateUserProfileInput,
    ) -> Result<bool> {
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        // Parse URLs if provided
        let avatar_url = if let Some(avatar) = input.avatar {
            if avatar.is_empty() {
                Some(None)
            } else {
                use url::Url;
                let url: Url = avatar.parse().map_err(|_| TinyBoardsError::from_message(400, "Invalid avatar URL"))?;
                Some(Some(url.into()))
            }
        } else {
            None
        };

        let banner_url = if let Some(banner) = input.banner {
            if banner.is_empty() {
                Some(None)
            } else {
                use url::Url;
                let url: Url = banner.parse().map_err(|_| TinyBoardsError::from_message(400, "Invalid banner URL"))?;
                Some(Some(url.into()))
            }
        } else {
            None
        };

        let background_url = if let Some(background) = input.profile_background {
            if background.is_empty() {
                Some(None)
            } else {
                use url::Url;
                let url: Url = background.parse().map_err(|_| TinyBoardsError::from_message(400, "Invalid background URL"))?;
                Some(Some(url.into()))
            }
        } else {
            None
        };

        let signature_url = if let Some(signature) = input.signature {
            if signature.is_empty() {
                Some(None)
            } else {
                use url::Url;
                let url: Url = signature.parse().map_err(|_| TinyBoardsError::from_message(400, "Invalid signature URL"))?;
                Some(Some(url.into()))
            }
        } else {
            None
        };

        // Process bio if provided
        let (bio, bio_html) = if let Some(bio_text) = input.bio {
            if bio_text.is_empty() {
                (None, None)
            } else {
                // Basic HTML processing - could be enhanced with proper markdown processing
                let html = bio_text.clone(); // For now, just store as-is
                (Some(bio_text), Some(html))
            }
        } else {
            (None, None)
        };

        let user_form = UserForm {
            display_name: input.display_name,
            bio,
            bio_html,
            avatar: avatar_url,
            banner: banner_url,
            profile_background: background_url,
            signature: signature_url,
            profile_music_youtube: Some(input.profile_music_youtube),
            ..UserForm::default()
        };

        DbUser::update(pool, user.id, &user_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update profile"))?;

        Ok(true)
    }

    /// Deactivate user account (set as deleted)
    pub async fn deactivate_account(&self, ctx: &Context<'_>) -> Result<bool> {
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        // Prevent admins from deactivating their own accounts
        if user.is_admin {
            return Err(TinyBoardsError::from_message(400, "Admin accounts cannot be deactivated").into());
        }

        let user_form = UserForm {
            is_deleted: Some(true),
            ..UserForm::default()
        };

        DbUser::update(pool, user.id, &user_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to deactivate account"))?;

        Ok(true)
    }

    /// Upload new avatar image
    pub async fn upload_avatar(
        &self,
        ctx: &Context<'_>,
        avatar: Upload,
    ) -> Result<String> {
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        // Delete the current avatar file if it exists
        if let Some(ref current_avatar) = user.avatar {
            use crate::helpers::files::upload::delete_file;
            delete_file(pool, current_avatar).await.ok(); // Don't fail if deletion fails
        }

        // Upload the new avatar with size limit of 5MB
        let avatar_url = upload_file(avatar, None, user.id, Some(5), ctx).await?;

        // Update user's avatar in database
        let user_form = UserForm {
            avatar: Some(Some(avatar_url.clone().into())),
            ..UserForm::default()
        };

        DbUser::update(pool, user.id, &user_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update avatar"))?;

        Ok(avatar_url.to_string())
    }

    /// Toggle bot account status
    pub async fn toggle_bot_account(
        &self,
        ctx: &Context<'_>,
        is_bot: bool,
    ) -> Result<bool> {
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        let user_form = UserForm {
            bot_account: Some(is_bot),
            ..UserForm::default()
        };

        DbUser::update(pool, user.id, &user_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update bot account status"))?;

        Ok(true)
    }

    /// Update notification preferences
    pub async fn update_notification_settings(
        &self,
        ctx: &Context<'_>,
        email_notifications_enabled: Option<bool>,
        show_bots: Option<bool>,
        show_nsfw: Option<bool>,
    ) -> Result<bool> {
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        let user_form = UserForm {
            email_notifications_enabled,
            show_bots,
            show_nsfw,
            ..UserForm::default()
        };

        DbUser::update(pool, user.id, &user_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update notification settings"))?;

        Ok(true)
    }

    /// Update interface preferences
    pub async fn update_interface_settings(
        &self,
        ctx: &Context<'_>,
        theme: Option<String>,
        interface_language: Option<String>,
        default_sort_type: Option<i16>,
        default_listing_type: Option<i16>,
    ) -> Result<bool> {
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        // Validate theme if provided
        if let Some(ref theme_name) = theme {
            let valid_themes = vec!["browser", "light", "dark", "pink", "blue", "green"];
            if !valid_themes.contains(&theme_name.as_str()) {
                return Err(TinyBoardsError::from_message(400, "Invalid theme").into());
            }
        }

        // Validate language if provided
        if let Some(ref lang) = interface_language {
            let valid_languages = vec!["en", "es", "fr", "de", "pt", "ru"];
            if !valid_languages.contains(&lang.as_str()) {
                return Err(TinyBoardsError::from_message(400, "Invalid language").into());
            }
        }

        let user_form = UserForm {
            theme,
            interface_language,
            default_sort_type,
            default_listing_type,
            ..UserForm::default()
        };

        DbUser::update(pool, user.id, &user_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update interface settings"))?;

        Ok(true)
    }

    /// Remove avatar image
    pub async fn remove_avatar(&self, ctx: &Context<'_>) -> Result<bool> {
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        // Check if user actually has an avatar to remove
        if user.avatar.is_none() {
            return Ok(true); // Already no avatar, nothing to do
        }

        // Delete the current avatar file if it exists
        if let Some(ref avatar) = user.avatar {
            use crate::helpers::files::upload::delete_file;
            delete_file(pool, avatar).await.ok(); // Don't fail if deletion fails
        }

        let user_form = UserForm {
            avatar: Some(None), // Explicitly set to None to signal a change
            ..UserForm::default()
        };

        DbUser::update(pool, user.id, &user_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to remove avatar"))?;

        Ok(true)
    }

    /// Remove banner image
    pub async fn remove_banner(&self, ctx: &Context<'_>) -> Result<bool> {
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        // Check if user actually has a banner to remove
        if user.banner.is_none() {
            return Ok(true); // Already no banner, nothing to do
        }

        // Delete the current banner file if it exists
        if let Some(ref banner) = user.banner {
            use crate::helpers::files::upload::delete_file;
            delete_file(pool, banner).await.ok(); // Don't fail if deletion fails
        }

        let user_form = UserForm {
            banner: Some(None), // Explicitly set to None to signal a change
            ..UserForm::default()
        };

        DbUser::update(pool, user.id, &user_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to remove banner"))?;

        Ok(true)
    }

    /// Remove profile background image
    pub async fn remove_profile_background(&self, ctx: &Context<'_>) -> Result<bool> {
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        // Delete the current background file if it exists
        if let Some(ref background) = user.profile_background {
            use crate::helpers::files::upload::delete_file;
            delete_file(pool, background).await.ok(); // Don't fail if deletion fails
        }

        let user_form = UserForm {
            profile_background: None,
            ..UserForm::default()
        };

        DbUser::update(pool, user.id, &user_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to remove profile background"))?;

        Ok(true)
    }

    /// Clear bio/about text
    pub async fn clear_bio(&self, ctx: &Context<'_>) -> Result<bool> {
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        // Check if user actually has a bio to clear
        if user.bio.is_none() || user.bio.as_ref().map_or(true, |b| b.trim().is_empty()) {
            return Ok(true); // Already no bio, nothing to do
        }

        let user_form = UserForm {
            bio: None,
            bio_html: None,
            ..UserForm::default()
        };

        DbUser::update(pool, user.id, &user_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to clear bio"))?;

        Ok(true)
    }
}