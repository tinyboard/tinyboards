use async_graphql::*;
use tinyboards_db::{
    aggregates::structs::BoardAggregates,
    models::{
        board::{
            board_mods::{BoardModerator, BoardModeratorForm, ModPerms},
            boards::Board,
        },
        user::user::{AdminPerms, User as DbUser},
    },
    traits::Crud,
    utils::{DbPool, get_conn},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

use crate::{structs::boards::Board as GqlBoard, LoggedInUser};

#[derive(Default)]
pub struct BoardModerationMutations;

#[derive(SimpleObject)]
pub struct AddModeratorResponse {
    pub success: bool,
    pub board: GqlBoard,
}

#[derive(SimpleObject)]
pub struct BanUserResponse {
    pub success: bool,
}

#[Object]
impl BoardModerationMutations {
    /// Add a moderator to a board
    pub async fn add_moderator(
        &self,
        ctx: &Context<'_>,
        board_id: i32,
        user_id: i32,
        permissions: Option<i32>,
    ) -> Result<AddModeratorResponse> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        // Verify board exists
        let board = Board::read(pool, board_id).await?;
        
        if board.is_banned || board.is_removed {
            return Err(TinyBoardsError::from_message(
                400,
                "Cannot add moderators to banned or removed boards",
            )
            .into());
        }

        // Check if user has permission to add moderators
        let can_add_mod = if user.has_permission(AdminPerms::Users) {
            true // Admins can add mods to any board
        } else {
            // Check if user is a moderator with Users permission on this board
            use tinyboards_db::schema::board_mods;
            let mod_entry: Option<BoardModerator> = board_mods::table
                .filter(board_mods::board_id.eq(board_id))
                .filter(board_mods::user_id.eq(user.id))
                .first(conn)
                .await
                .optional()?;

            match mod_entry {
                Some(mod_entry) => {
                    // Check if user has Users (16) or Full (32) permissions
                    let permissions = mod_entry.permissions;
                    permissions & (ModPerms::Users.as_i32() | ModPerms::Full.as_i32()) != 0
                }
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

        // Verify target user exists
        DbUser::read(pool, user_id).await?;

        // Check if user is already a moderator
        use tinyboards_db::schema::board_mods;
        let existing_mod = board_mods::table
            .filter(board_mods::board_id.eq(board_id))
            .filter(board_mods::user_id.eq(user_id))
            .first::<BoardModerator>(conn)
            .await
            .optional()?;

        if existing_mod.is_some() {
            return Err(TinyBoardsError::from_message(
                400,
                "User is already a moderator of this board",
            )
            .into());
        }

        // Get next rank
        let max_rank: Option<i32> = board_mods::table
            .filter(board_mods::board_id.eq(board_id))
            .select(diesel::dsl::max(board_mods::rank))
            .first(conn)
            .await?;

        let next_rank = max_rank.unwrap_or(0) + 1;

        let form = BoardModeratorForm {
            board_id: Some(board_id),
            user_id: Some(user_id),
            permissions: permissions.or(Some(ModPerms::Content as i32)),
            rank: Some(next_rank),
            invite_accepted: Some(true),
            invite_accepted_date: Some(Some(chrono::Utc::now().naive_utc())),
        };

        BoardModerator::create(pool, &form).await?;

        // For simplicity, we'll just return success without the full board details for now
        // TODO: Implement proper board aggregates loading
        let updated_board = Board::read(pool, board_id).await?;
        let aggregates = BoardAggregates {
            id: updated_board.id,
            board_id: updated_board.id,
            subscribers: 0,
            posts: 0, 
            comments: 0,
            creation_date: updated_board.creation_date,
            users_active_day: 0,
            users_active_week: 0,
            users_active_month: 0,
            users_active_half_year: 0,
        };
        let gql_board = GqlBoard::from((updated_board, aggregates));

        Ok(AddModeratorResponse {
            success: true,
            board: gql_board,
        })
    }

    /// Ban a user from a board (site-wide ban, admin only)
    pub async fn ban_user(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
        _reason: Option<String>,
        expires: Option<String>, // ISO date string
    ) -> Result<BanUserResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        // Only admins can perform site-wide bans
        if !user.has_permission(AdminPerms::Users) {
            return Err(TinyBoardsError::from_message(
                403,
                "Only admins can ban users site-wide",
            )
            .into());
        }

        // Verify target user exists and isn't already banned
        let target_user = DbUser::read(pool, user_id).await?;
        if target_user.is_banned {
            return Err(TinyBoardsError::from_message(
                400,
                "User is already banned",
            )
            .into());
        }

        // Don't allow banning other admins
        if target_user.is_admin {
            return Err(TinyBoardsError::from_message(
                400,
                "Cannot ban other administrators",
            )
            .into());
        }

        // Parse expires date if provided
        let expires_date = if let Some(expires_str) = expires {
            Some(chrono::NaiveDateTime::parse_from_str(&expires_str, "%Y-%m-%dT%H:%M:%S")
                .map_err(|_| TinyBoardsError::from_message(400, "Invalid expires date format"))?)
        } else {
            None
        };

        // Update user to banned status
        use tinyboards_db::schema::users;
        diesel::update(users::table.find(user_id))
            .set((
                users::is_banned.eq(true),
                users::unban_date.eq(expires_date),
            ))
            .execute(&mut get_conn(pool).await?)
            .await?;

        // TODO: Log the ban action with reason

        Ok(BanUserResponse { success: true })
    }
}