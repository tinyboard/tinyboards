use crate::{LoggedInUser, structs::boards::Board};
use async_graphql::*;
use tinyboards_db::{
    models::board::{
        boards::{Board as DbBoard, BoardForm},
        board_mods::{BoardModerator, BoardModeratorForm},
        board_subscriber::{BoardSubscriber, BoardSubscriberForm},
    },
    models::site::site::Site,
    traits::{Crud, Subscribeable, Joinable},
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;
use chrono::Utc;

#[derive(InputObject)]
pub struct CreateBoardInput {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub is_nsfw: Option<bool>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub hover_color: Option<String>,
}

#[derive(SimpleObject)]
pub struct CreateBoardResponse {
    pub board: Board,
}

#[derive(Default)]
pub struct CreateBoard;

#[Object]
impl CreateBoard {
    /// Create a new board
    async fn create_board(
        &self,
        ctx: &Context<'_>,
        input: CreateBoardInput,
    ) -> Result<CreateBoardResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Check if board creation is allowed
        let site = Site::read(pool).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to read site settings"))?;

        // Check if user can create boards
        let admin_level = user.admin_level;
        if site.board_creation_admin_only && admin_level == 0 {
            return Err(TinyBoardsError::from_message(403, "Board creation is restricted to admins").into());
        }

        if !user.board_creation_approved && admin_level == 0 {
            return Err(TinyBoardsError::from_message(403, "Board creation requires approval").into());
        }

        // Validate board name - basic validation for now
        if input.name.is_empty() || input.name.len() > 50 || input.name.contains(' ') {
            return Err(TinyBoardsError::from_message(400, "Invalid board name").into());
        }

        // Check if board name already exists
        if DbBoard::get_by_name(pool, &input.name).await.is_ok() {
            return Err(TinyBoardsError::from_message(400, "Board name already exists").into());
        }

        let board_form = BoardForm {
            name: Some(input.name),
            title: Some(input.title),
            description: Some(input.description),
            is_nsfw: input.is_nsfw,
            primary_color: input.primary_color.or(Some("#1976d2".to_string())),
            secondary_color: input.secondary_color.or(Some("#424242".to_string())),
            hover_color: input.hover_color.or(Some("#1565c0".to_string())),
            last_refreshed_date: Some(Utc::now().naive_utc()),
            ..Default::default()
        };

        // Create the board
        let db_board = DbBoard::create(pool, &board_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to create board"))?;

        // Add creator as moderator
        let mod_form = BoardModeratorForm {
            board_id: Some(db_board.id),
            user_id: Some(user.id),
            permissions: Some(8191), // All permissions
            rank: Some(1), // Top rank
            invite_accepted: Some(true),
            invite_accepted_date: Some(Some(Utc::now().naive_utc())),
        };

        BoardModerator::join(pool, &mod_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to add creator as moderator"))?;

        // Subscribe creator to the board
        let sub_form = BoardSubscriberForm {
            board_id: db_board.id,
            user_id: user.id,
            pending: Some(false),
        };

        BoardSubscriber::subscribe(pool, &sub_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to subscribe creator to board"))?;

        // Get board with counts for the GraphQL response
        let board_with_counts = DbBoard::get_with_counts_for_ids(pool, vec![db_board.id]).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to read board with counts"))?
            .into_iter()
            .next()
            .ok_or_else(|| TinyBoardsError::from_message(500, "Failed to get created board with counts"))?;

        let board = Board::from(board_with_counts);

        Ok(CreateBoardResponse { board })
    }
}