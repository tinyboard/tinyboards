use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbModerationAction,
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        moderator::moderation_log::ModerationLogInsertForm,
        social::{BoardUserBan, BoardUserBanInsertForm},
        user::user::{AdminPerms, User as DbUser},
    },
    schema::{board_moderators, board_user_bans, moderation_log, users},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::LoggedInUser;

#[derive(Default)]
pub struct BoardBanMutations;

#[derive(SimpleObject)]
pub struct BoardBanResponse {
    pub success: bool,
    pub ban_id: ID,
    pub message: String,
}

#[derive(SimpleObject)]
pub struct BoardUnbanResponse {
    pub success: bool,
    pub message: String,
}

#[derive(InputObject)]
pub struct BoardBanUserInput {
    pub user_id: ID,
    pub board_id: ID,
    pub reason: Option<String>,
    pub expires_days: Option<i32>,
}

#[Object]
impl BoardBanMutations {
    /// Ban a user from a specific board (moderator/admin only)
    pub async fn ban_user_from_board(
        &self,
        ctx: &Context<'_>,
        input: BoardBanUserInput,
    ) -> Result<BoardBanResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let target_id: Uuid = input.user_id.parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid user ID".into()))?;
        let board_id: Uuid = input.board_id.parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        // Check permissions
        let is_admin = user.has_permission(AdminPerms::Content);
        if !is_admin {
            let moderator: BoardModerator = board_moderators::table
                .filter(board_moderators::board_id.eq(board_id))
                .filter(board_moderators::user_id.eq(user.id))
                .filter(board_moderators::is_invite_accepted.eq(true))
                .first(conn)
                .await
                .map_err(|_| TinyBoardsError::from_message(403, "You are not a moderator of this board"))?;

            if !moderator.has_permission(ModPerms::Users) {
                return Err(TinyBoardsError::from_message(403, "You don't have permission to ban users from this board").into());
            }
        }

        if user.id == target_id {
            return Err(TinyBoardsError::from_message(400, "You cannot ban yourself").into());
        }

        let target_user: DbUser = users::table
            .find(target_id)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("User not found".into()))?;

        // Prevent banning other moderators unless admin
        if !is_admin {
            let is_target_mod = board_moderators::table
                .filter(board_moderators::board_id.eq(board_id))
                .filter(board_moderators::user_id.eq(target_id))
                .filter(board_moderators::is_invite_accepted.eq(true))
                .count()
                .get_result::<i64>(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?
                > 0;

            if is_target_mod {
                return Err(TinyBoardsError::from_message(403, "Cannot ban other moderators from this board").into());
            }
        }

        // Check if already banned
        let already_banned = board_user_bans::table
            .filter(board_user_bans::board_id.eq(board_id))
            .filter(board_user_bans::user_id.eq(target_id))
            .filter(
                board_user_bans::expires_at.is_null()
                    .or(board_user_bans::expires_at.gt(chrono::Utc::now()))
            )
            .count()
            .get_result::<i64>(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?
            > 0;

        if already_banned {
            return Err(TinyBoardsError::from_message(400, "User is already banned from this board").into());
        }

        let expires_at = input.expires_days.map(|days| {
            chrono::Utc::now() + chrono::Duration::days(days as i64)
        });

        let ban_form = BoardUserBanInsertForm {
            board_id,
            user_id: target_id,
            expires_at,
        };

        let ban: BoardUserBan = diesel::insert_into(board_user_bans::table)
            .values(&ban_form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Log the moderation action
        let metadata = serde_json::json!({
            "target_username": target_user.name,
            "board_id": board_id.to_string(),
            "is_permanent": expires_at.is_none(),
        });

        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: user.id,
                action_type: DbModerationAction::BanFromBoard,
                target_type: "user".to_string(),
                target_id,
                board_id: Some(board_id),
                reason: input.reason,
                metadata: Some(metadata),
                expires_at,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let message = if expires_at.is_some() {
            format!("User {} has been temporarily banned from the board", target_user.name)
        } else {
            format!("User {} has been permanently banned from the board", target_user.name)
        };

        Ok(BoardBanResponse {
            success: true,
            ban_id: ban.id.to_string().into(),
            message,
        })
    }

    /// Unban a user from a specific board (moderator/admin only)
    pub async fn unban_user_from_board(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        board_id: ID,
        reason: Option<String>,
    ) -> Result<BoardUnbanResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let target_id: Uuid = user_id.parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid user ID".into()))?;
        let board_uuid: Uuid = board_id.parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        // Check permissions
        let is_admin = user.has_permission(AdminPerms::Content);
        if !is_admin {
            let moderator: BoardModerator = board_moderators::table
                .filter(board_moderators::board_id.eq(board_uuid))
                .filter(board_moderators::user_id.eq(user.id))
                .filter(board_moderators::is_invite_accepted.eq(true))
                .first(conn)
                .await
                .map_err(|_| TinyBoardsError::from_message(403, "You are not a moderator of this board"))?;

            if !moderator.has_permission(ModPerms::Users) {
                return Err(TinyBoardsError::from_message(403, "You don't have permission to unban users from this board").into());
            }
        }

        let target_user: DbUser = users::table
            .find(target_id)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("User not found".into()))?;

        let rows_affected = diesel::delete(
            board_user_bans::table
                .filter(board_user_bans::board_id.eq(board_uuid))
                .filter(board_user_bans::user_id.eq(target_id))
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if rows_affected == 0 {
            return Err(TinyBoardsError::from_message(400, "User is not currently banned from this board").into());
        }

        // Log the moderation action
        let metadata = serde_json::json!({
            "target_username": target_user.name,
            "board_id": board_uuid.to_string(),
            "unbanned_by": user.name,
        });

        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: user.id,
                action_type: DbModerationAction::UnbanFromBoard,
                target_type: "user".to_string(),
                target_id,
                board_id: Some(board_uuid),
                reason,
                metadata: Some(metadata),
                expires_at: None,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(BoardUnbanResponse {
            success: true,
            message: format!("User {} has been unbanned from the board", target_user.name),
        })
    }
}
