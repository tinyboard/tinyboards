use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    board::{EditBoard, BoardResponse, BoardIdPath},
    utils::{
        blocking,
        require_user,
    },
};
use tinyboards_db::{
    models::{
        board::boards::{Board, BoardForm},
    },
    traits::Crud, utils::naive_now,
};
use tinyboards_db_views::structs::BoardView;
use tinyboards_utils::{
    parser::parse_markdown,
    TinyBoardsError,
};

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

        let title = data.title.clone();
        let mut description = data.description.clone();
        let is_nsfw = data.is_nsfw.clone();

        // board update restricted to board mod or admin (may provide other options in the future)
        let _user = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(path.board_id.clone(), context.pool())
            .await
            .unwrap()?;

        if let Some(desc) = description {
            description = parse_markdown(&desc);
        }

        let form = BoardForm {
            title,
            description: Some(description),
            is_nsfw,
            updated: Some(Some(naive_now())),
            ..BoardForm::default()
        };

        // update the board
        let board = blocking(context.pool(), move |conn| {
            Board::update(conn, path.board_id.clone(), &form)
        })
        .await??;

        let board_view = blocking(context.pool(), move |conn| {
            BoardView::read(conn, board.id, None)
        })
        .await??;

        Ok(BoardResponse { board_view })
    }
}