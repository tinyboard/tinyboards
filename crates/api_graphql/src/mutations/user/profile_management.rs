/**
 * Additional Profile Management Operations
 */
use crate::{DbPool, LoggedInUser};
use async_graphql::*;
use tinyboards_db::models::user::user::{User as DbUser, UserForm};
use tinyboards_db::traits::Crud;
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct ProfileManagement;

#[Object]
impl ProfileManagement {
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

        // Delete the current avatar file if it exists
        if let Some(ref avatar) = user.avatar {
            use crate::helpers::files::upload::delete_file;
            delete_file(pool, avatar).await.ok(); // Don't fail if deletion fails
        }

        let user_form = UserForm {
            avatar: None,
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

        // Delete the current banner file if it exists
        if let Some(ref banner) = user.banner {
            use crate::helpers::files::upload::delete_file;
            delete_file(pool, banner).await.ok(); // Don't fail if deletion fails
        }

        let user_form = UserForm {
            banner: None,
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