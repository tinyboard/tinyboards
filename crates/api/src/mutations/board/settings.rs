use crate::{LoggedInUser, structs::boards::Board};
use async_graphql::*;
use tinyboards_db::{
    models::board::{
        boards::{Board as DbBoard, BoardForm},
        board_mods::BoardModerator,
    },
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;
use chrono::Utc;

#[derive(InputObject)]
pub struct UpdateBoardSettingsInput {
    pub board_id: i32,
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_nsfw: Option<bool>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub hover_color: Option<String>,
    pub sidebar: Option<String>,
    pub posting_restricted_to_mods: Option<bool>,
    pub is_hidden: Option<bool>,
    pub exclude_from_all: Option<bool>,
    pub icon: Option<String>,
    pub banner: Option<String>,
}

#[derive(SimpleObject)]
pub struct UpdateBoardSettingsResponse {
    pub board: Board,
}

#[derive(Default)]
pub struct UpdateBoardSettings;

#[Object]
impl UpdateBoardSettings {
    /// Update board settings
    async fn update_board_settings(
        &self,
        ctx: &Context<'_>,
        input: UpdateBoardSettingsInput,
    ) -> Result<UpdateBoardSettingsResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Check if board exists
        let _board = DbBoard::read(pool, input.board_id).await
            .map_err(|_| TinyBoardsError::from_message(404, "Board not found"))?;

        // Check if user is a moderator of this board or admin
        let admin_level = user.admin_level;
        let is_admin = admin_level > 0;
        let is_mod = if !is_admin {
            // Check if user is a moderator by trying to get the mod relationship
            BoardModerator::get_by_user_id_for_board(pool, user.id, input.board_id, true).await.is_ok()
        } else {
            true
        };

        if !is_mod && !is_admin {
            return Err(TinyBoardsError::from_message(403, "You must be a moderator or admin to update board settings").into());
        }

        // Build the update form
        let board_form = BoardForm {
            title: input.title,
            description: Some(input.description),
            is_nsfw: input.is_nsfw,
            primary_color: input.primary_color,
            secondary_color: input.secondary_color,
            hover_color: input.hover_color,
            sidebar: Some(input.sidebar),
            posting_restricted_to_mods: input.posting_restricted_to_mods,
            is_hidden: input.is_hidden,
            exclude_from_all: input.exclude_from_all,
            icon: input.icon.and_then(|s| s.parse::<url::Url>().ok().map(|url| url.into())),
            banner: input.banner.and_then(|s| s.parse::<url::Url>().ok().map(|url| url.into())),
            updated: Some(Some(Utc::now().naive_utc())),
            ..Default::default()
        };

        // Update the board
        let _updated_db_board = DbBoard::update(pool, input.board_id, &board_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update board settings"))?;

        // Get board with counts for the GraphQL response
        let board_with_counts = DbBoard::get_with_counts_for_ids(pool, vec![input.board_id]).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to read board with counts"))?
            .into_iter()
            .next()
            .ok_or_else(|| TinyBoardsError::from_message(500, "Failed to get updated board with counts"))?;

        let board = Board::from(board_with_counts);

        Ok(UpdateBoardSettingsResponse { board })
    }
}