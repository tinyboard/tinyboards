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
}