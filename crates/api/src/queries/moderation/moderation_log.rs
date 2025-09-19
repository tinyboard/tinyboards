use async_graphql::*;
use tinyboards_db::{
    models::{
        moderator::moderation_log::ModerationLog,
        board::board_mods::{BoardModerator, ModPerms},
        user::user::{AdminPerms, User},
    },
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

use crate::LoggedInUser;

#[derive(Default)]
pub struct ModerationLogQueries;

#[derive(SimpleObject)]
pub struct ModerationLogEntry {
    pub id: i32,
    pub moderator_id: i32,
    pub moderator_name: String,
    pub action_type: String,
    pub target_type: String,
    pub target_id: i32,
    pub board_id: Option<i32>,
    pub reason: Option<String>,
    pub metadata: Option<String>, // JSON stringified
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(SimpleObject)]
pub struct ModerationLogResponse {
    pub entries: Vec<ModerationLogEntry>,
    pub total_count: i32,
}

impl From<(ModerationLog, String)> for ModerationLogEntry {
    fn from((log, moderator_name): (ModerationLog, String)) -> Self {
        Self {
            id: log.id,
            moderator_id: log.moderator_id,
            moderator_name,
            action_type: log.action_type,
            target_type: log.target_type,
            target_id: log.target_id,
            board_id: log.board_id,
            reason: log.reason,
            metadata: log.metadata.map(|v| v.to_string()),
            created_at: log.created_at.to_string(),
            expires_at: log.expires_at.map(|dt| dt.to_string()),
        }
    }
}

#[Object]
impl ModerationLogQueries {
    /// Get moderation log (admin/moderator only)
    pub async fn get_moderation_log(
        &self,
        ctx: &Context<'_>,
        board_id: Option<i32>,
        action_type: Option<String>,
        moderator_id: Option<i32>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<ModerationLogResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        // Check if user is admin or moderator
        let is_admin = user.has_permission(AdminPerms::Content);

        if !is_admin && board_id.is_none() {
            return Err(TinyBoardsError::from_message(
                403,
                "You must specify a board_id if you're not an admin",
            )
            .into());
        }

        // If board_id is specified, check if user is a moderator of that board
        if let Some(board_id) = board_id {
            if !is_admin {
                match BoardModerator::get_by_user_id_for_board(pool, user.id, board_id, true).await {
                    Ok(moderator) => {
                        if !moderator.has_permission(ModPerms::Content) {
                            return Err(TinyBoardsError::from_message(
                                403,
                                "You don't have permission to view the moderation log for this board",
                            )
                            .into());
                        }
                    }
                    Err(_) => {
                        return Err(TinyBoardsError::from_message(
                            403,
                            "You are not a moderator of this board",
                        )
                        .into());
                    }
                }
            }
        }

        // Get moderation log entries
        let logs = ModerationLog::list(
            pool,
            board_id,
            action_type,
            moderator_id,
            Some(limit),
            Some(offset),
        ).await?;

        // Get moderator names for each entry
        let mut entries = Vec::new();
        for log in logs {
            match User::read(pool, log.moderator_id).await {
                Ok(moderator) => {
                    entries.push(ModerationLogEntry::from((log, moderator.name)));
                }
                Err(_) => {
                    // If moderator was deleted, use "Deleted User"
                    entries.push(ModerationLogEntry::from((log, "Deleted User".to_string())));
                }
            }
        }

        // Get total count for pagination
        let total_count = ModerationLog::list(
            pool,
            board_id,
            None,
            moderator_id,
            None,
            None,
        ).await?.len() as i32;

        Ok(ModerationLogResponse {
            entries,
            total_count,
        })
    }

    /// Get moderation actions for a specific target (admin/moderator only)
    pub async fn get_target_moderation_history(
        &self,
        ctx: &Context<'_>,
        target_type: String,
        target_id: i32,
        limit: Option<i32>,
    ) -> Result<Vec<ModerationLogEntry>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        // Check if user has admin or moderator permissions
        let is_admin = user.has_permission(AdminPerms::Content);

        if !is_admin {
            // For non-admins, we need to check if they're a moderator of the relevant board
            // This would require getting the board_id from the target, which is complex
            // For now, let's require admin permissions for target history
            return Err(TinyBoardsError::from_message(
                403,
                "Admin privileges required to view target moderation history",
            )
            .into());
        }

        let limit = limit.unwrap_or(50).min(100);

        // Get all moderation log entries for this target
        let logs = ModerationLog::list(
            pool,
            None, // All boards
            None, // All action types
            None, // All moderators
            Some(limit),
            Some(0),
        ).await?;

        // Filter by target
        let filtered_logs: Vec<ModerationLog> = logs.into_iter()
            .filter(|log| log.target_type == target_type && log.target_id == target_id)
            .collect();

        // Get moderator names for each entry
        let mut entries = Vec::new();
        for log in filtered_logs {
            match User::read(pool, log.moderator_id).await {
                Ok(moderator) => {
                    entries.push(ModerationLogEntry::from((log, moderator.name)));
                }
                Err(_) => {
                    entries.push(ModerationLogEntry::from((log, "Deleted User".to_string())));
                }
            }
        }

        Ok(entries)
    }
}