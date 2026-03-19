use crate::{helpers::permissions, structs::boards::Board};
use async_graphql::*;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbModerationAction,
    models::{
        aggregates::BoardAggregates as DbBoardAggregates,
        board::{
            board_mods::{BoardModerator, BoardModeratorInsertForm, ModPerms},
            boards::{Board as DbBoard, BoardUpdateForm},
        },
        moderator::moderation_log::ModerationLogInsertForm,
        user::user::AdminPerms,
    },
    schema::{board_aggregates, board_moderators, boards, moderation_log},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

#[derive(Default)]
pub struct AdminBoardModeration;

#[Object]
impl AdminBoardModeration {
    /// Ban a board (admin only) - permanent ban with public reason
    pub async fn admin_ban_board(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
        public_reason: String,
        admin_notes: Option<String>,
    ) -> Result<Board> {
        let pool = ctx.data::<DbPool>()?;
        let user = permissions::require_admin_permission(ctx, AdminPerms::Boards)?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid board ID".to_string()))?;

        let conn = &mut get_conn(pool).await?;

        // Load the board and verify it is not already banned
        let board: DbBoard = boards::table
            .find(board_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found".to_string()))?;

        if board.is_banned {
            return Err(TinyBoardsError::from_message(400, "Board is already banned").into());
        }

        // Apply the ban
        let ban_form = BoardUpdateForm {
            is_banned: Some(true),
            public_ban_reason: Some(Some(public_reason.clone())),
            ban_reason: admin_notes.map(Some),
            banned_by: Some(Some(user.id)),
            banned_at: Some(Some(Utc::now())),
            ..Default::default()
        };

        diesel::update(boards::table.find(board_uuid))
            .set(&ban_form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to ban board: {}", e)))?;

        // Log the action
        let log_form = ModerationLogInsertForm {
            moderator_id: user.id,
            action_type: DbModerationAction::RemoveBoard,
            target_type: "board".to_string(),
            target_id: board_uuid,
            board_id: Some(board_uuid),
            reason: Some(public_reason),
            metadata: None,
            expires_at: None,
        };

        diesel::insert_into(moderation_log::table)
            .values(&log_form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to log board ban: {}", e)))?;

        // Return updated board with aggregates
        let updated_board: DbBoard = boards::table
            .find(board_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found after ban".to_string()))?;

        let agg: Option<DbBoardAggregates> = board_aggregates::table
            .filter(board_aggregates::board_id.eq(board_uuid))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(Board::from_db(updated_board, agg))
    }

    /// Unban a board (admin only)
    pub async fn admin_unban_board(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
    ) -> Result<Board> {
        let pool = ctx.data::<DbPool>()?;
        let user = permissions::require_admin_permission(ctx, AdminPerms::Boards)?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid board ID".to_string()))?;

        let conn = &mut get_conn(pool).await?;

        // Load the board and verify it is banned
        let board: DbBoard = boards::table
            .find(board_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found".to_string()))?;

        if !board.is_banned {
            return Err(TinyBoardsError::from_message(400, "Board is not banned").into());
        }

        // Remove the ban
        let unban_form = BoardUpdateForm {
            is_banned: Some(false),
            public_ban_reason: Some(None),
            ban_reason: Some(None),
            banned_by: Some(None),
            banned_at: Some(None),
            ..Default::default()
        };

        diesel::update(boards::table.find(board_uuid))
            .set(&unban_form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to unban board: {}", e)))?;

        // Log the action
        let log_form = ModerationLogInsertForm {
            moderator_id: user.id,
            action_type: DbModerationAction::RestoreBoard,
            target_type: "board".to_string(),
            target_id: board_uuid,
            board_id: Some(board_uuid),
            reason: None,
            metadata: None,
            expires_at: None,
        };

        diesel::insert_into(moderation_log::table)
            .values(&log_form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to log board unban: {}", e)))?;

        // Return updated board with aggregates
        let updated_board: DbBoard = boards::table
            .find(board_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found after unban".to_string()))?;

        let agg: Option<DbBoardAggregates> = board_aggregates::table
            .filter(board_aggregates::board_id.eq(board_uuid))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(Board::from_db(updated_board, agg))
    }

    /// Get list of banned boards (admin only)
    pub async fn admin_banned_boards(&self, ctx: &Context<'_>) -> Result<Vec<Board>> {
        let pool = ctx.data::<DbPool>()?;
        let _user = permissions::require_admin_permission(ctx, AdminPerms::Boards)?;

        let conn = &mut get_conn(pool).await?;

        let banned: Vec<DbBoard> = boards::table
            .filter(boards::is_banned.eq(true))
            .filter(boards::deleted_at.is_null())
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to fetch banned boards: {}", e)))?;

        let board_ids: Vec<Uuid> = banned.iter().map(|b| b.id).collect();

        let aggs: Vec<DbBoardAggregates> = board_aggregates::table
            .filter(board_aggregates::board_id.eq_any(&board_ids))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let boards: Vec<Board> = banned
            .into_iter()
            .map(|b| {
                let agg = aggs.iter().find(|a| a.board_id == b.id).cloned();
                Board::from_db(b, agg)
            })
            .collect();

        Ok(boards)
    }

    /// Exclude a board from the global feed (/all) - admin only
    pub async fn exclude_board_from_all(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
        exclude: bool,
    ) -> Result<Board> {
        let pool = ctx.data::<DbPool>()?;
        let _user = permissions::require_admin_permission(ctx, AdminPerms::Boards)?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid board ID".to_string()))?;

        let conn = &mut get_conn(pool).await?;

        // Verify board exists
        let _board: DbBoard = boards::table
            .find(board_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found".to_string()))?;

        let update_form = BoardUpdateForm {
            exclude_from_all: Some(exclude),
            ..Default::default()
        };

        diesel::update(boards::table.find(board_uuid))
            .set(&update_form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to update board: {}", e)))?;

        let updated_board: DbBoard = boards::table
            .find(board_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found after update".to_string()))?;

        let agg: Option<DbBoardAggregates> = board_aggregates::table
            .filter(board_aggregates::board_id.eq(board_uuid))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(Board::from_db(updated_board, agg))
    }

    /// Add admin as moderator to any board (admin only)
    pub async fn admin_add_self_as_mod(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
        mod_perms: Option<i32>,
    ) -> Result<Board> {
        let pool = ctx.data::<DbPool>()?;
        let user = permissions::require_admin_permission(ctx, AdminPerms::Boards)?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid board ID".to_string()))?;

        let conn = &mut get_conn(pool).await?;

        // Verify board exists
        let _board: DbBoard = boards::table
            .find(board_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found".to_string()))?;

        // Check if admin is already a moderator
        let existing_mod: Option<BoardModerator> = board_moderators::table
            .filter(board_moderators::board_id.eq(board_uuid))
            .filter(board_moderators::user_id.eq(user.id))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if existing_mod.is_some() {
            return Err(
                TinyBoardsError::from_message(400, "You are already a moderator of this board")
                    .into(),
            );
        }

        // Place admin above the current top-ranked moderator
        let highest_rank: Option<i32> = board_moderators::table
            .filter(board_moderators::board_id.eq(board_uuid))
            .select(diesel::dsl::min(board_moderators::rank))
            .first(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let top_rank = highest_rank.unwrap_or(1);
        let admin_rank = if top_rank > 0 { top_rank - 1 } else { 0 };

        let mod_permissions = mod_perms.unwrap_or(ModPerms::Full.as_bitmask());

        let mod_form = BoardModeratorInsertForm {
            board_id: board_uuid,
            user_id: user.id,
            permissions: mod_permissions,
            rank: admin_rank,
            is_invite_accepted: true,
            invite_accepted_at: Some(Utc::now()),
        };

        diesel::insert_into(board_moderators::table)
            .values(&mod_form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to add admin as moderator: {}", e)))?;

        let updated_board: DbBoard = boards::table
            .find(board_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found after update".to_string()))?;

        let agg: Option<DbBoardAggregates> = board_aggregates::table
            .filter(board_aggregates::board_id.eq(board_uuid))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(Board::from_db(updated_board, agg))
    }

    /// Remove admin from board moderation (admin only)
    pub async fn admin_remove_self_as_mod(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
    ) -> Result<Board> {
        let pool = ctx.data::<DbPool>()?;
        let user = permissions::require_admin_permission(ctx, AdminPerms::Boards)?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid board ID".to_string()))?;

        let conn = &mut get_conn(pool).await?;

        // Verify board exists
        let _board: DbBoard = boards::table
            .find(board_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found".to_string()))?;

        // Check if admin is actually a moderator
        let existing_mod: Option<BoardModerator> = board_moderators::table
            .filter(board_moderators::board_id.eq(board_uuid))
            .filter(board_moderators::user_id.eq(user.id))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if existing_mod.is_none() {
            return Err(
                TinyBoardsError::from_message(400, "You are not a moderator of this board").into(),
            );
        }

        diesel::delete(
            board_moderators::table
                .filter(board_moderators::board_id.eq(board_uuid))
                .filter(board_moderators::user_id.eq(user.id)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(format!("Failed to remove admin as moderator: {}", e)))?;

        let updated_board: DbBoard = boards::table
            .find(board_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found after update".to_string()))?;

        let agg: Option<DbBoardAggregates> = board_aggregates::table
            .filter(board_aggregates::board_id.eq(board_uuid))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(Board::from_db(updated_board, agg))
    }
}
