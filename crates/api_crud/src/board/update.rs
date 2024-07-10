use crate::PerformCrud;
use actix_web::web::Data;
use regex::Regex;
use tinyboards_api_common::{
    board::{BoardIdPath, BoardResponse, EditBoard},
    build_response::build_board_response,
    data::TinyBoardsContext,
    utils::require_user,
};
use tinyboards_db::models::board::board_mods::ModPerms;
use tinyboards_db::{
    models::board::boards::{Board, BoardForm},
    traits::Crud,
    utils::naive_now,
};
use tinyboards_utils::{parser::parse_markdown_opt, utils::custom_body_parsing, TinyBoardsError};

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for EditBoard {
    type Response = BoardResponse;
    type Route = BoardIdPath;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<BoardResponse, TinyBoardsError> {
        let data: &EditBoard = &self;

        let board = Board::read(context.pool(), path.board_id.clone()).await?;

        let name = data.name.clone();
        let title = data.title.clone();
        let icon = data.icon.clone();
        let banner = data.banner.clone();
        let description = data.description.clone();
        let is_nsfw = data.is_nsfw.clone();
        let primary_color = data.primary_color.clone();
        let secondary_color = data.secondary_color.clone();
        let hover_color = data.hover_color.clone();

        // board update restricted to board mod or admin (may provide other options in the future)
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(
                context.pool(),
                path.board_id.clone(),
                ModPerms::Config,
                None,
            )
            .await
            .unwrap()?;

        let name = name.unwrap_or(board.name.clone());
        let description = match description {
            Some(desc) => Some(desc),
            None => board.description,
        };
        let icon = match icon {
            Some(icon) => Some(icon),
            None => board.icon,
        };
        let banner = match banner {
            Some(banner) => Some(banner),
            None => board.banner,
        };
        let primary_color = match primary_color {
            Some(pc) => Some(pc),
            None => Some(board.primary_color),
        };
        let secondary_color = match secondary_color {
            Some(sc) => Some(sc),
            None => Some(board.secondary_color),
        };
        let hover_color = match hover_color {
            Some(hc) => Some(hc),
            None => Some(board.hover_color),
        };

        // default board's name may be changed entirely, for everything else, only the capitalization can be updated
        if !(board.id == 1 || board.name.to_lowercase() == name.to_lowercase()) {
            return Err(TinyBoardsError::from_message(
                400,
                "Only the capitalization of the board name can be changed.",
            ));
        }

        // Check name
        let re = Regex::new(r"^[A-Za-z0-9][A-Za-z0-9_]{0,29}$").unwrap();
        if !re.is_match(&name) {
            return Err(TinyBoardsError::from_message(400, "Board name contains disallowed characters. Allowed: alphanumerics and underscores, except as the first character."));
        }

        // Description length
        if let Some(ref desc) = description {
            if desc.len() > 255 {
                return Err(TinyBoardsError::from_message(
                    400,
                    "Description cannot be longer than 255 characters.",
                ));
            }
        }

        // Sidebar!
        let sidebar = match data.sidebar.clone() {
            Some(s) => Some(s),
            None => board.sidebar,
        };

        let sidebar_html = if let Some(ref sidebar) = data.sidebar {
            let sidebar_html = parse_markdown_opt(sidebar);
            Some(custom_body_parsing(
                &sidebar_html.unwrap_or_default(),
                context.settings(),
            ))
        } else {
            board.sidebar_html
        };

        // if let Some(desc) = description {
        //     description = parse_markdown_opt(&desc);
        // }

        let form = BoardForm {
            name: Some(name),
            title,
            description: Some(description),
            is_nsfw,
            icon,
            banner,
            primary_color,
            secondary_color,
            hover_color,
            sidebar: Some(sidebar),
            sidebar_html: Some(sidebar_html),
            updated: Some(Some(naive_now())),
            ..BoardForm::default()
        };

        // update the board
        let board = Board::update(context.pool(), path.board_id.clone(), &form).await?;

        Ok(build_board_response(context, view, board.id).await?)
    }
}
