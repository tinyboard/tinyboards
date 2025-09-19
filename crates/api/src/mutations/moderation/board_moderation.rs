use async_graphql::*;
use serde_json;
use tinyboards_db::{
    models::{
        board::board_user_bans::{BoardUserBan, BoardUserBanForm},
        board::board_mods::{BoardModerator, ModPerms},
        user::user::{AdminPerms, User},
        moderator::moderation_log::{ModerationLog, action_types, target_types},
    },
    traits::{Crud, Bannable},
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

use crate::LoggedInUser;

#[derive(Default)]
pub struct BoardModerationMutations;

#[derive(SimpleObject)]
pub struct BoardBanResponse {
    pub success: bool,
    pub ban_id: i32,
    pub message: String,
}

#[derive(SimpleObject)]
pub struct BoardUnbanResponse {
    pub success: bool,
    pub message: String,
}

#[derive(InputObject)]
pub struct BoardBanUserInput {
    pub user_id: i32,
    pub board_id: i32,
    pub reason: Option<String>,
    pub expires_days: Option<i32>, // Number of days until ban expires, None for permanent
}

#[Object]
impl BoardModerationMutations {
    /// Ban a user from a specific board (moderator/admin only)
    pub async fn ban_user_from_board(
        &self,
        ctx: &Context<'_>,
        input: BoardBanUserInput,
    ) -> Result<BoardBanResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        // Check if user has permission to ban from this board
        let is_admin = user.has_permission(AdminPerms::Content);
        let is_moderator = if !is_admin {
            match BoardModerator::get_by_user_id_for_board(pool, user.id, input.board_id, true).await {
                Ok(moderator) => moderator.has_permission(ModPerms::Users),
                Err(_) => false,
            }
        } else {
            true
        };

        if !is_admin && !is_moderator {
            return Err(TinyBoardsError::from_message(
                403,
                "You don't have permission to ban users from this board",
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

        // Prevent banning other moderators/admins unless you're an admin
        if !is_admin {
            match BoardModerator::get_by_user_id_for_board(pool, input.user_id, input.board_id, true).await {
                Ok(_) => {
                    return Err(TinyBoardsError::from_message(
                        403,
                        "Cannot ban other moderators from this board",
                    )
                    .into());
                }
                Err(_) => {} // User is not a moderator, OK to ban
            }
        }

        // Check if user is already banned from this board
        if BoardUserBan::is_banned(pool, input.user_id, input.board_id).await? {
            return Err(TinyBoardsError::from_message(
                400,
                "User is already banned from this board",
            )
            .into());
        }

        // Calculate expiration date if temporary ban
        let expires_at = input.expires_days.map(|days| {
            chrono::Utc::now().naive_utc() + chrono::Duration::days(days as i64)
        });

        // Create the board ban
        let form = BoardUserBanForm {
            board_id: Some(input.board_id),
            user_id: Some(input.user_id),
            creation_date: Some(chrono::Utc::now().naive_utc()),
            expires: expires_at,
        };

        let ban = BoardUserBan::create(pool, &form).await?;

        // Log the moderation action
        let metadata = serde_json::json!({
            "target_username": target_user.name,
            "board_id": input.board_id,
            "expires_at": expires_at,
            "is_permanent": expires_at.is_none(),
        });

        ModerationLog::log_action(
            pool,
            user.id,
            action_types::BAN_FROM_BOARD,
            target_types::USER,
            input.user_id,
            Some(input.board_id),
            input.reason,
            Some(metadata),
            expires_at,
        ).await?;

        let message = if expires_at.is_some() {
            format!("User {} has been temporarily banned from the board", target_user.name)
        } else {
            format!("User {} has been permanently banned from the board", target_user.name)
        };

        Ok(BoardBanResponse {
            success: true,
            ban_id: ban.id,
            message,
        })
    }

    /// Unban a user from a specific board (moderator/admin only)
    pub async fn unban_user_from_board(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
        board_id: i32,
        reason: Option<String>,
    ) -> Result<BoardUnbanResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        // Check if user has permission to unban from this board
        let is_admin = user.has_permission(AdminPerms::Content);
        let is_moderator = if !is_admin {
            match BoardModerator::get_by_user_id_for_board(pool, user.id, board_id, true).await {
                Ok(moderator) => moderator.has_permission(ModPerms::Users),
                Err(_) => false,
            }
        } else {
            true
        };

        if !is_admin && !is_moderator {
            return Err(TinyBoardsError::from_message(
                403,
                "You don't have permission to unban users from this board",
            )
            .into());
        }

        // Check if target user exists
        let target_user = User::read(pool, user_id).await?;

        // Check if user is actually banned from this board
        if !BoardUserBan::is_banned(pool, user_id, board_id).await? {
            return Err(TinyBoardsError::from_message(
                400,
                "User is not currently banned from this board",
            )
            .into());
        }

        // Remove the board ban
        let rows_affected = BoardUserBan::unban_user(pool, user_id, board_id).await?;

        if rows_affected == 0 {
            return Err(TinyBoardsError::from_message(
                400,
                "Failed to remove board ban",
            )
            .into());
        }

        // Log the moderation action
        let metadata = serde_json::json!({
            "target_username": target_user.name,
            "board_id": board_id,
            "unbanned_by": user.name,
        });

        ModerationLog::log_action(
            pool,
            user.id,
            action_types::UNBAN_FROM_BOARD,
            target_types::USER,
            user_id,
            Some(board_id),
            reason,
            Some(metadata),
            None,
        ).await?;

        Ok(BoardUnbanResponse {
            success: true,
            message: format!("User {} has been unbanned from the board", target_user.name),
        })
    }
}