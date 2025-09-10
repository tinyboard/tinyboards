use crate::{DbPool, LoggedInUser, structs::boards::Board};
use async_graphql::*;
use tinyboards_db::{
    models::{
        board::boards::{Board as DbBoard, BoardForm},
        board::board_mods::{BoardModerator as DbBoardMod, BoardModeratorForm, ModPerms},
        person::local_user::AdminPerms,
        moderator::admin_actions::AdminBanBoard,
    },
    traits::{Crud, Joinable},
    utils::{DbPool as DbPoolTrait, naive_now},
};
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct AdminBoardModeration;

#[Object]
impl AdminBoardModeration {
    /// Ban a board (admin only) - permanent ban with public reason
    pub async fn admin_ban_board(
        &self,
        ctx: &Context<'_>,
        board_id: i32,
        public_reason: String,
        admin_notes: Option<String>,
    ) -> Result<Board> {
        let user = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        // Check admin permissions
        if !user.has_permission(AdminPerms::Boards) {
            return Err(TinyBoardsError::from_message(403, "Admin permissions required").into());
        }

        // Get the board
        let board = DbBoard::read(pool, board_id)
            .await
            .map_err(|_| TinyBoardsError::from_message(404, "Board not found"))?;

        // Check if already banned
        if board.is_banned {
            return Err(TinyBoardsError::from_message(400, "Board is already banned").into());
        }

        // Ban the board
        board
            .admin_ban(
                pool, 
                user.person.id, 
                &public_reason,
                admin_notes.as_deref()
            )
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to ban board"))?;

        // Return updated board
        let updated_boards = DbBoard::get_with_counts_for_ids(pool, vec![board_id])
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to fetch updated board"))?;
            
        let updated_board = updated_boards.into_iter().next()
            .ok_or_else(|| TinyBoardsError::from_message(404, "Board not found after update"))?;

        Ok(Board::from(updated_board))
    }

    /// Unban a board (admin only)
    pub async fn admin_unban_board(
        &self,
        ctx: &Context<'_>,
        board_id: i32,
    ) -> Result<Board> {
        let user = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        // Check admin permissions
        if !user.has_permission(AdminPerms::Boards) {
            return Err(TinyBoardsError::from_message(403, "Admin permissions required").into());
        }

        // Get the board
        let board = DbBoard::read(pool, board_id)
            .await
            .map_err(|_| TinyBoardsError::from_message(404, "Board not found"))?;

        // Check if not banned
        if !board.is_banned {
            return Err(TinyBoardsError::from_message(400, "Board is not banned").into());
        }

        // Unban the board
        board
            .admin_unban(pool, user.person.id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to unban board"))?;

        // Return updated board
        let updated_boards = DbBoard::get_with_counts_for_ids(pool, vec![board_id])
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to fetch updated board"))?;
            
        let updated_board = updated_boards.into_iter().next()
            .ok_or_else(|| TinyBoardsError::from_message(404, "Board not found after update"))?;

        Ok(Board::from(updated_board))
    }

    /// Get list of banned boards (admin only)
    pub async fn admin_banned_boards(&self, ctx: &Context<'_>) -> Result<Vec<Board>> {
        let user = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        // Check admin permissions
        if !user.has_permission(AdminPerms::Boards) {
            return Err(TinyBoardsError::from_message(403, "Admin permissions required").into());
        }

        // Get banned boards
        let banned_boards = DbBoard::get_banned_boards_with_counts(pool)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to fetch banned boards"))?;

        Ok(banned_boards.into_iter().map(Board::from).collect())
    }

    /// Get ban history for a board (admin only)
    pub async fn board_ban_history(
        &self,
        ctx: &Context<'_>,
        target_board_id: i32,
    ) -> Result<Vec<AdminBanBoardResult>> {
        let user = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        // Check admin permissions
        if !user.has_permission(AdminPerms::Boards) {
            return Err(TinyBoardsError::from_message(403, "Admin permissions required").into());
        }

        // Get ban history
        let history = DbBoard::get_ban_history(pool, target_board_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to fetch ban history"))?;

        Ok(history.into_iter().map(AdminBanBoardResult::from).collect())
    }

    /// Exclude a board from the global feed (/all) - admin only
    pub async fn exclude_board_from_all(
        &self,
        ctx: &Context<'_>,
        board_id: i32,
        exclude: bool,
    ) -> Result<Board> {
        let user = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        // Check admin permissions
        if !user.has_permission(AdminPerms::Boards) {
            return Err(TinyBoardsError::from_message(403, "Admin permissions required").into());
        }

        // Get the board
        let _board = DbBoard::read(pool, board_id)
            .await
            .map_err(|_| TinyBoardsError::from_message(404, "Board not found"))?;

        // Update the exclude_from_all setting
        let update_form = BoardForm {
            exclude_from_all: Some(exclude),
            ..BoardForm::default()
        };

        DbBoard::update(pool, board_id, &update_form)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update board"))?;

        // Return updated board
        let updated_boards = DbBoard::get_with_counts_for_ids(pool, vec![board_id])
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to fetch updated board"))?;
            
        let updated_board = updated_boards.into_iter().next()
            .ok_or_else(|| TinyBoardsError::from_message(404, "Board not found after update"))?;

        Ok(Board::from(updated_board))
    }

    /// Add admin as moderator to any board (admin only)
    pub async fn admin_add_self_as_mod(
        &self,
        ctx: &Context<'_>,
        board_id: i32,
        permissions: Option<i32>,
    ) -> Result<Board> {
        let user = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        // Check admin permissions
        if !user.has_permission(AdminPerms::Boards) {
            return Err(TinyBoardsError::from_message(403, "Admin permissions required").into());
        }

        // Get the board to verify it exists
        let _board = DbBoard::read(pool, board_id)
            .await
            .map_err(|_| TinyBoardsError::from_message(404, "Board not found"))?;

        // Check if admin is already a moderator
        let existing_mod = DbBoardMod::get_by_person_id_for_board(pool, user.person.id, board_id, false).await;
        
        if existing_mod.is_ok() {
            return Err(TinyBoardsError::from_message(400, "You are already a moderator of this board").into());
        }

        // Get the highest rank (lowest number) to place admin at the top
        let existing_mods = DbBoardMod::for_board(pool, board_id).await.unwrap_or_default();
        let highest_rank = existing_mods.iter().map(|m| m.rank).min().unwrap_or(1);
        let admin_rank = if highest_rank > 1 { highest_rank - 1 } else { 0 };

        // Set permissions - default to Full if not specified
        let mod_permissions = permissions.unwrap_or(ModPerms::Full.as_i32());

        // Create moderator relationship
        let mod_form = BoardModeratorForm {
            board_id: Some(board_id),
            person_id: Some(user.person.id),
            permissions: Some(mod_permissions),
            rank: Some(admin_rank),
            invite_accepted: Some(true), // Admin doesn't need to accept invite
            invite_accepted_date: Some(Some(naive_now())),
        };

        DbBoardMod::join(pool, &mod_form)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to add admin as moderator"))?;

        // Return updated board
        let updated_boards = DbBoard::get_with_counts_for_ids(pool, vec![board_id])
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to fetch updated board"))?;
            
        let updated_board = updated_boards.into_iter().next()
            .ok_or_else(|| TinyBoardsError::from_message(404, "Board not found after update"))?;

        Ok(Board::from(updated_board))
    }

    /// Remove admin from board moderation (admin only)
    pub async fn admin_remove_self_as_mod(
        &self,
        ctx: &Context<'_>,
        board_id: i32,
    ) -> Result<Board> {
        let user = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        // Check admin permissions
        if !user.has_permission(AdminPerms::Boards) {
            return Err(TinyBoardsError::from_message(403, "Admin permissions required").into());
        }

        // Get the board to verify it exists
        let _board = DbBoard::read(pool, board_id)
            .await
            .map_err(|_| TinyBoardsError::from_message(404, "Board not found"))?;

        // Check if admin is actually a moderator
        let existing_mod = DbBoardMod::get_by_person_id_for_board(pool, user.person.id, board_id, false).await;
        
        if existing_mod.is_err() {
            return Err(TinyBoardsError::from_message(400, "You are not a moderator of this board").into());
        }

        // Remove moderator relationship
        DbBoardMod::remove_board_mod(pool, user.person.id, board_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to remove admin as moderator"))?;

        // Return updated board
        let updated_boards = DbBoard::get_with_counts_for_ids(pool, vec![board_id])
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to fetch updated board"))?;
            
        let updated_board = updated_boards.into_iter().next()
            .ok_or_else(|| TinyBoardsError::from_message(404, "Board not found after update"))?;

        Ok(Board::from(updated_board))
    }
}

/// GraphQL-friendly representation of ban history
#[derive(SimpleObject)]
pub struct AdminBanBoardResult {
    pub id: i32,
    pub admin_id: i32,
    pub board_id: i32,
    pub internal_notes: Option<String>,
    pub public_ban_reason: Option<String>,
    pub action: String,
    pub when_: String,
}

impl From<AdminBanBoard> for AdminBanBoardResult {
    fn from(ban: AdminBanBoard) -> Self {
        Self {
            id: ban.id,
            admin_id: ban.admin_id,
            board_id: ban.board_id,
            internal_notes: ban.internal_notes,
            public_ban_reason: ban.public_ban_reason,
            action: ban.action,
            when_: ban.when_.to_string(),
        }
    }
}