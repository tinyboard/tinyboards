use crate::{LoggedInUser, structs::boards::Board, helpers::files::upload::upload_file, Settings};
use async_graphql::*;
use tinyboards_db::{
    models::board::{
        boards::{Board as DbBoard, BoardForm},
        board_mods::BoardModerator,
    },
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::{TinyBoardsError, parser::parse_markdown_opt, utils::custom_body_parsing};
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
        icon_file: Option<Upload>,
        banner_file: Option<Upload>,
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

        let settings = ctx.data::<Settings>()?.as_ref();

        // Handle file uploads
        let icon_url = match icon_file {
            Some(file) => Some(upload_file(file, None, user.id, Some(settings.media.max_board_icon_size_mb), ctx).await?.to_string()),
            None => input.icon
        };

        let banner_url = match banner_file {
            Some(file) => Some(upload_file(file, None, user.id, Some(settings.media.max_board_banner_size_mb), ctx).await?.to_string()),
            None => input.banner
        };

        // Parse sidebar markdown to HTML if sidebar is being updated
        let sidebar_html = match &input.sidebar {
            Some(sidebar_text) => {
                let sidebar_html = parse_markdown_opt(sidebar_text);
                Some(Some(custom_body_parsing(&sidebar_html.unwrap_or_default(), settings)))
            }
            None => None,
        };

        // Build the update form - only include fields that were provided to avoid data loss
        let board_form = BoardForm {
            title: input.title,
            description: input.description.map(Some), // Only update if provided
            is_nsfw: input.is_nsfw,
            primary_color: input.primary_color,
            secondary_color: input.secondary_color,
            hover_color: input.hover_color,
            sidebar: input.sidebar.map(Some), // Only update if provided
            sidebar_html, // Update sidebar_html when sidebar is updated
            posting_restricted_to_mods: input.posting_restricted_to_mods,
            is_hidden: input.is_hidden,
            exclude_from_all: input.exclude_from_all,
            icon: icon_url.and_then(|s| s.parse::<url::Url>().ok().map(|url| url.into())),
            banner: banner_url.and_then(|s| s.parse::<url::Url>().ok().map(|url| url.into())),
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