use async_graphql::*;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        aggregates::BoardAggregates as DbBoardAggregates,
        board::{
            board_mods::{BoardModerator, BoardModeratorInsertForm, ModPerms},
            boards::Board as DbBoard,
        },
        user::user::{AdminPerms, User as DbUser},
    },
    schema::{board_aggregates, board_moderators, boards, users},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::structs::boards::Board as GqlBoard;

#[derive(Default)]
pub struct BoardModerationMutations;

#[derive(SimpleObject)]
pub struct AddModeratorResponse {
    pub success: bool,
    pub board: GqlBoard,
}

#[derive(SimpleObject)]
pub struct RemoveModeratorResponse {
    pub success: bool,
    pub message: String,
}

#[derive(SimpleObject)]
pub struct TransferOwnershipResponse {
    pub success: bool,
    pub message: String,
}

#[Object]
impl BoardModerationMutations {
    /// Add a moderator to a board
    pub async fn add_moderator(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
        user_id: ID,
        permissions: Option<i32>,
    ) -> Result<AddModeratorResponse> {
        let pool = ctx.data::<DbPool>()?;
        let caller = crate::helpers::permissions::require_auth_not_banned(ctx)?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid board ID".to_string()))?;
        let target_uuid: Uuid = user_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid user ID".to_string()))?;

        let conn = &mut get_conn(pool).await?;

        // Verify the board exists and is not banned or removed
        let board: DbBoard = boards::table
            .find(board_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found".to_string()))?;

        if board.is_banned || board.is_removed {
            return Err(TinyBoardsError::from_message(
                400,
                "Cannot add moderators to banned or removed boards",
            )
            .into());
        }

        // Check if the caller has permission to add moderators:
        // site admins with Users permission can always do this;
        // otherwise the caller must be a board mod with Users permission.
        let can_add_mod = if caller.has_permission(AdminPerms::Users) {
            true
        } else {
            let mod_entry: Option<BoardModerator> = board_moderators::table
                .filter(board_moderators::board_id.eq(board_uuid))
                .filter(board_moderators::user_id.eq(caller.id))
                .first(conn)
                .await
                .optional()
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            match mod_entry {
                Some(m) => m.has_permission(ModPerms::Users),
                None => false,
            }
        };

        if !can_add_mod {
            return Err(TinyBoardsError::from_message(
                403,
                "You don't have permission to add moderators to this board",
            )
            .into());
        }

        // Verify the target user exists
        let _target: DbUser = users::table
            .find(target_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("User not found".to_string()))?;

        // Check if target is already a moderator
        let existing: Option<BoardModerator> = board_moderators::table
            .filter(board_moderators::board_id.eq(board_uuid))
            .filter(board_moderators::user_id.eq(target_uuid))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if existing.is_some() {
            return Err(TinyBoardsError::from_message(
                400,
                "User is already a moderator of this board",
            )
            .into());
        }

        // Assign next available rank (append to end of mod list)
        let max_rank: Option<i32> = board_moderators::table
            .filter(board_moderators::board_id.eq(board_uuid))
            .select(diesel::dsl::max(board_moderators::rank))
            .first(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let next_rank = max_rank.unwrap_or(0) + 1;

        let form = BoardModeratorInsertForm {
            board_id: board_uuid,
            user_id: target_uuid,
            permissions: permissions.unwrap_or(ModPerms::Content.as_bitmask()),
            rank: next_rank,
            is_invite_accepted: true,
            invite_accepted_at: Some(Utc::now()),
        };

        diesel::insert_into(board_moderators::table)
            .values(&form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to add moderator: {}", e)))?;

        // Re-fetch board + aggregates for the response
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

        let gql_board = GqlBoard::from_db(updated_board, agg);

        Ok(AddModeratorResponse {
            success: true,
            board: gql_board,
        })
    }

    /// Remove a moderator from a board (board owner or admin only)
    pub async fn remove_board_moderator(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
        user_id: ID,
    ) -> Result<RemoveModeratorResponse> {
        let pool = ctx.data::<DbPool>()?;
        let caller = crate::helpers::permissions::require_auth_not_banned(ctx)?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid board ID".to_string()))?;
        let target_uuid: Uuid = user_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid user ID".to_string()))?;

        let conn = &mut get_conn(pool).await?;

        // Verify board exists
        let _board: DbBoard = boards::table
            .find(board_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found".to_string()))?;

        // Check caller permission: admins can remove any mod; board owner (rank 0) can remove mods
        let can_remove = if caller.has_permission(AdminPerms::Users) {
            true
        } else {
            let caller_mod: Option<BoardModerator> = board_moderators::table
                .filter(board_moderators::board_id.eq(board_uuid))
                .filter(board_moderators::user_id.eq(caller.id))
                .first(conn)
                .await
                .optional()
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            matches!(caller_mod, Some(m) if m.rank == 0)
        };

        if !can_remove {
            return Err(TinyBoardsError::from_message(
                403,
                "Only board owners or admins can remove moderators",
            )
            .into());
        }

        // Prevent self-removal unless admin
        if caller.id == target_uuid && !caller.has_permission(AdminPerms::Users) {
            return Err(TinyBoardsError::from_message(
                400,
                "Board owners cannot remove themselves. Transfer ownership first.",
            )
            .into());
        }

        // Fetch target mod entry
        let target_mod: BoardModerator = board_moderators::table
            .filter(board_moderators::board_id.eq(board_uuid))
            .filter(board_moderators::user_id.eq(target_uuid))
            .first(conn)
            .await
            .map_err(|_| {
                TinyBoardsError::NotFound("User is not a moderator of this board".to_string())
            })?;

        // Prevent removing the board owner unless admin
        if target_mod.rank == 0 && !caller.has_permission(AdminPerms::Users) {
            return Err(TinyBoardsError::from_message(
                403,
                "Cannot remove board owner. Transfer ownership first.",
            )
            .into());
        }

        diesel::delete(board_moderators::table.find(target_mod.id))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to remove moderator: {}", e)))?;

        let target_user: DbUser = users::table
            .find(target_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("User not found".to_string()))?;

        Ok(RemoveModeratorResponse {
            success: true,
            message: format!("User {} has been removed as a moderator", target_user.name),
        })
    }

    /// Transfer board ownership to another moderator (board owner or admin only)
    pub async fn transfer_board_ownership(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
        new_owner_id: ID,
    ) -> Result<TransferOwnershipResponse> {
        let pool = ctx.data::<DbPool>()?;
        let caller = crate::helpers::permissions::require_auth_not_banned(ctx)?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid board ID".to_string()))?;
        let new_owner_uuid: Uuid = new_owner_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid user ID".to_string()))?;

        let conn = &mut get_conn(pool).await?;

        // Verify board exists
        let _board: DbBoard = boards::table
            .find(board_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found".to_string()))?;

        // Only the current owner (rank 0) or a site admin may transfer ownership
        let can_transfer = if caller.has_permission(AdminPerms::Users) {
            true
        } else {
            let caller_mod: Option<BoardModerator> = board_moderators::table
                .filter(board_moderators::board_id.eq(board_uuid))
                .filter(board_moderators::user_id.eq(caller.id))
                .first(conn)
                .await
                .optional()
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            matches!(caller_mod, Some(m) if m.rank == 0)
        };

        if !can_transfer {
            return Err(TinyBoardsError::from_message(
                403,
                "Only board owners or admins can transfer ownership",
            )
            .into());
        }

        // Fetch the prospective new owner's mod entry — they must already be a moderator
        let new_owner_mod: BoardModerator = board_moderators::table
            .filter(board_moderators::board_id.eq(board_uuid))
            .filter(board_moderators::user_id.eq(new_owner_uuid))
            .first(conn)
            .await
            .map_err(|_| {
                TinyBoardsError::from_message(
                    400,
                    "New owner must be a moderator of this board",
                )
            })?;

        if new_owner_mod.rank == 0 {
            return Err(TinyBoardsError::from_message(
                400,
                "User is already the owner of this board",
            )
            .into());
        }

        // Promote new owner to rank 0 with full permissions
        diesel::update(board_moderators::table.find(new_owner_mod.id))
            .set((
                board_moderators::rank.eq(0),
                board_moderators::permissions.eq(ModPerms::Full.as_bitmask()),
            ))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to promote new owner: {}", e)))?;

        // Demote the previous owner to rank 1 (if caller is not a site admin)
        if !caller.has_permission(AdminPerms::Users) {
            let caller_mod: Option<BoardModerator> = board_moderators::table
                .filter(board_moderators::board_id.eq(board_uuid))
                .filter(board_moderators::user_id.eq(caller.id))
                .first(conn)
                .await
                .optional()
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            if let Some(current_mod) = caller_mod {
                diesel::update(board_moderators::table.find(current_mod.id))
                    .set(board_moderators::rank.eq(1))
                    .execute(conn)
                    .await
                    .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
            }
        }

        let new_owner: DbUser = users::table
            .find(new_owner_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("User not found".to_string()))?;

        Ok(TransferOwnershipResponse {
            success: true,
            message: format!(
                "Board ownership has been transferred to {}",
                new_owner.name
            ),
        })
    }
}
