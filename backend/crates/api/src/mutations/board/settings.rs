use crate::{
    helpers::{files::upload::upload_file_opendal, permissions},
    structs::boards::Board,
    Settings,
};
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbBoardMode,
    models::{
        aggregates::BoardAggregates as DbBoardAggregates,
        board::boards::{Board as DbBoard, BoardUpdateForm},
        board::board_mods::ModPerms,
        user::user::AdminPerms,
    },
    schema::{board_aggregates, boards},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::{
    css_sanitizer::{sanitize_css, MAX_BOARD_CSS_BYTES},
    parser::parse_markdown_opt,
    utils::custom_body_parsing,
    TinyBoardsError,
};
use url::Url;
use uuid::Uuid;

#[derive(InputObject)]
pub struct UpdateBoardSettingsInput {
    pub board_id: ID,
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
    /// Board mode: "feed" or "forum".
    pub mode: Option<String>,
    pub wiki_enabled: Option<bool>,
    pub custom_css: Option<String>,
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

        let board_uuid: Uuid = input
            .board_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid board ID".to_string()))?;

        // Require the caller to be a board mod with Config permission, or a boards-capable admin
        let user = permissions::require_board_mod_or_admin(
            ctx,
            pool,
            board_uuid,
            ModPerms::Config,
            Some(AdminPerms::Boards),
        )
        .await?;

        let settings = ctx.data::<Settings>()?.as_ref();

        // Parse and validate board mode if provided
        let board_mode = match input.mode.as_deref() {
            Some("feed") => Some(DbBoardMode::Feed),
            Some("forum") => Some(DbBoardMode::Forum),
            Some(other) => {
                return Err(TinyBoardsError::from_message(
                    400,
                    &format!("Invalid board mode '{}'. Must be 'feed' or 'forum'.", other),
                )
                .into());
            }
            None => None,
        };

        // Handle file uploads
        let icon_url = match icon_file {
            Some(file) => Some(
                upload_file_opendal(
                    file,
                    None,
                    user.id,
                    Some(settings.media.max_board_icon_size_mb),
                    ctx,
                )
                .await?
                .to_string(),
            ),
            None => input.icon,
        };

        let banner_url = match banner_file {
            Some(file) => Some(
                upload_file_opendal(
                    file,
                    None,
                    user.id,
                    Some(settings.media.max_board_banner_size_mb),
                    ctx,
                )
                .await?
                .to_string(),
            ),
            None => input.banner,
        };

        // Parse sidebar markdown to HTML when the sidebar is being updated
        let sidebar_html = match &input.sidebar {
            Some(sidebar_text) => {
                let html = parse_markdown_opt(sidebar_text);
                Some(Some(custom_body_parsing(
                    &html.unwrap_or_default(),
                    settings,
                )))
            }
            None => None,
        };

        // Sanitize custom CSS if provided
        let sanitized_css = match input.custom_css {
            Some(ref css) if !css.trim().is_empty() => {
                let result = sanitize_css(css, MAX_BOARD_CSS_BYTES)
                    .map_err(|e| TinyBoardsError::BadRequest(e))?;
                Some(Some(result.css))
            }
            Some(_) => Some(None), // Empty string clears the CSS
            None => None,
        };

        // Build the update form — only fields that were provided are set, preserving existing data
        let board_form = BoardUpdateForm {
            title: input.title,
            description: input.description.map(Some),
            is_nsfw: input.is_nsfw,
            primary_color: input.primary_color,
            secondary_color: input.secondary_color,
            hover_color: input.hover_color,
            sidebar: input.sidebar.map(Some),
            sidebar_html,
            is_posting_restricted_to_mods: input.posting_restricted_to_mods,
            is_hidden: input.is_hidden,
            exclude_from_all: input.exclude_from_all,
            icon: icon_url.map(|s| Url::parse(&s).ok().map(|u| u.to_string())),
            banner: banner_url.map(|s| Url::parse(&s).ok().map(|u| u.to_string())),
            mode: board_mode,
            wiki_enabled: input.wiki_enabled,
            custom_css: sanitized_css,
            ..Default::default()
        };

        let conn = &mut get_conn(pool).await?;

        // Apply the update
        diesel::update(boards::table.find(board_uuid))
            .set(&board_form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to update board settings: {}", e)))?;

        // Re-fetch the updated board and its aggregates
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

        let board = Board::from_db(updated_board, agg);

        Ok(UpdateBoardSettingsResponse { board })
    }
}
