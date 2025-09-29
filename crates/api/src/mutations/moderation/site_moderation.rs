use async_graphql::*;
use serde_json;
use tinyboards_db::{
    models::{
        user::{
            user::User,
            user_ban::{UserBan, UserBanForm},
            user::{AdminPerms},
        },
        moderator::moderation_log::{ModerationLog, action_types, target_types},
    },
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

use crate::LoggedInUser;

#[derive(Default)]
pub struct SiteModerationMutations;

#[derive(SimpleObject)]
pub struct BanUserResponse {
    pub success: bool,
    pub ban_id: i32,
    pub message: String,
}

#[derive(SimpleObject)]
pub struct UnbanUserResponse {
    pub success: bool,
    pub message: String,
}

#[derive(InputObject)]
pub struct BanUserInput {
    pub user_id: i32,
    pub reason: Option<String>,
    pub expires_days: Option<i32>, // Number of days until ban expires, None for permanent
}

#[derive(InputObject)]
pub struct SetAdminLevelInput {
    pub username: String,
    pub level: i32,
    pub reason: Option<String>,
}

#[derive(SimpleObject)]
pub struct SetAdminLevelResponse {
    pub success: bool,
    pub message: String,
}

#[Object]
impl SiteModerationMutations {
    /// Ban a user site-wide (admin only)
    pub async fn ban_user(
        &self,
        ctx: &Context<'_>,
        input: BanUserInput,
    ) -> Result<BanUserResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        // Check if user is admin with content permissions
        if !user.has_permission(AdminPerms::Content) {
            return Err(TinyBoardsError::from_message(
                403,
                "Admin privileges required to ban users site-wide",
            )
            .into());
        }

        // Prevent self-ban
        if user.id == input.user_id {
            return Err(TinyBoardsError::from_message(
                400,
                "You cannot ban yourself",
            )
            .into());
        }

        // Check if target user exists
        let target_user = User::read(pool, input.user_id).await?;

        // Prevent banning other admins
        if target_user.has_permission(AdminPerms::Content) {
            return Err(TinyBoardsError::from_message(
                403,
                "Cannot ban other administrators",
            )
            .into());
        }

        // Check if user is already banned
        if UserBan::is_user_banned(pool, input.user_id).await? {
            return Err(TinyBoardsError::from_message(
                400,
                "User is already banned",
            )
            .into());
        }

        // Calculate expiration date if temporary ban
        let expires_at = input.expires_days.map(|days| {
            chrono::Utc::now().naive_utc() + chrono::Duration::days(days as i64)
        });

        // Create the ban
        let ban = UserBan::ban_user(
            pool,
            input.user_id,
            user.id,
            input.reason.clone(),
            expires_at,
        ).await?;

        // Update user's banned status
        User::ban_user(pool, input.user_id, true).await?;

        // Log the moderation action
        let metadata = serde_json::json!({
            "target_username": target_user.name,
            "expires_at": expires_at,
            "is_permanent": expires_at.is_none(),
        });

        ModerationLog::log_action(
            pool,
            user.id,
            action_types::BAN_USER,
            target_types::USER,
            input.user_id,
            None, // Site-wide action
            input.reason,
            Some(metadata),
            expires_at,
        ).await?;

        let message = if expires_at.is_some() {
            format!("User {} has been temporarily banned", target_user.name)
        } else {
            format!("User {} has been permanently banned", target_user.name)
        };

        Ok(BanUserResponse {
            success: true,
            ban_id: ban.id,
            message,
        })
    }

    /// Unban a user site-wide (admin only)
    pub async fn unban_user(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
        reason: Option<String>,
    ) -> Result<UnbanUserResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        // Check if user is admin with content permissions
        if !user.has_permission(AdminPerms::Content) {
            return Err(TinyBoardsError::from_message(
                403,
                "Admin privileges required to unban users",
            )
            .into());
        }

        // Check if target user exists
        let target_user = User::read(pool, user_id).await?;

        // Check if user is actually banned
        if !UserBan::is_user_banned(pool, user_id).await? {
            return Err(TinyBoardsError::from_message(
                400,
                "User is not currently banned",
            )
            .into());
        }

        // Remove the ban
        let rows_affected = UserBan::unban_user(pool, user_id).await?;

        if rows_affected == 0 {
            return Err(TinyBoardsError::from_message(
                400,
                "Failed to remove ban",
            )
            .into());
        }

        // Update user's banned status
        User::ban_user(pool, user_id, false).await?;

        // Log the moderation action
        let metadata = serde_json::json!({
            "target_username": target_user.name,
            "unbanned_by": user.name,
        });

        ModerationLog::log_action(
            pool,
            user.id,
            action_types::UNBAN_USER,
            target_types::USER,
            user_id,
            None, // Site-wide action
            reason,
            Some(metadata),
            None,
        ).await?;

        Ok(UnbanUserResponse {
            success: true,
            message: format!("User {} has been unbanned", target_user.name),
        })
    }

    /// Set admin level for a user (high-level admin only)
    pub async fn set_admin_level(
        &self,
        ctx: &Context<'_>,
        input: SetAdminLevelInput,
    ) -> Result<SetAdminLevelResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        // Check if user has permission to manage other admins
        // Only Full, Owner, or System level admins can modify admin levels
        if !user.has_permission(AdminPerms::Full) &&
           !user.has_permission(AdminPerms::Owner) &&
           !user.has_permission(AdminPerms::System) {
            return Err(TinyBoardsError::from_message(
                403,
                "Full admin privileges or higher required to manage admin levels",
            )
            .into());
        }

        // Find target user by username
        let target_user = User::get_by_name(pool, input.username.clone()).await
            .map_err(|_| TinyBoardsError::from_message(404, "User not found"))?;

        // Prevent self-modification (for safety)
        if user.id == target_user.id {
            return Err(TinyBoardsError::from_message(
                400,
                "You cannot modify your own admin level",
            )
            .into());
        }

        // Prevent lower-level admins from modifying higher-level admins
        if target_user.admin_level >= user.admin_level && target_user.admin_level > 0 {
            return Err(TinyBoardsError::from_message(
                403,
                "Cannot modify admin level of someone with equal or higher privileges",
            )
            .into());
        }

        // Validate the requested admin level
        if input.level < 0 || input.level > 8 {
            return Err(TinyBoardsError::from_message(
                400,
                "Invalid admin level. Must be between 0-8",
            )
            .into());
        }

        // Prevent granting higher level than the requesting admin
        if input.level >= user.admin_level && user.admin_level < 8 {
            return Err(TinyBoardsError::from_message(
                403,
                "Cannot grant admin level equal to or higher than your own",
            )
            .into());
        }

        // Update the user's admin level
        User::update_admin(pool, target_user.id, input.level).await?;

        // Log the moderation action
        let action_type = if input.level > target_user.admin_level {
            "PROMOTE_ADMIN"
        } else if input.level < target_user.admin_level {
            "DEMOTE_ADMIN"
        } else {
            "UPDATE_ADMIN_LEVEL"
        };

        let metadata = serde_json::json!({
            "target_username": target_user.name,
            "old_level": target_user.admin_level,
            "new_level": input.level,
            "admin_username": user.name,
        });

        ModerationLog::log_action(
            pool,
            user.id,
            action_type,
            target_types::USER,
            target_user.id,
            None, // Site-wide action
            input.reason,
            Some(metadata),
            None,
        ).await?;

        let message = if input.level == 0 {
            format!("User {} has been removed as admin", target_user.name)
        } else if target_user.admin_level == 0 {
            format!("User {} has been promoted to admin (level {})", target_user.name, input.level)
        } else {
            format!("User {} admin level updated from {} to {}", target_user.name, target_user.admin_level, input.level)
        };

        Ok(SetAdminLevelResponse {
            success: true,
            message,
        })
    }
}