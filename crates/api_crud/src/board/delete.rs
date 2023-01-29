use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    board::{DeleteBoard, BoardIdPath, BoardResponse},
    utils::{
        blocking,
        require_user,
    },
};

use tinyboards_db::{
    models::{
        board::boards::Board,
    },
    traits::Crud, 
};
use tinyboards_db_views::structs::BoardView;
use tinyboards_utils::{
    TinyBoardsError,
};

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for DeleteBoard {
    type Response = BoardResponse;
    type Route = BoardIdPath;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<BoardResponse, TinyBoardsError> {

        // board delete restricted to admin (may provide other options in the future)
        let _user = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        let orig_board = blocking(context.pool(), move |conn| {
            Board::read(conn, path.board_id.clone())
        })
        .await??;

        // negate the deleted status from the board as it is currently in the database to get new deleted status
        let new_is_deleted = !orig_board.is_deleted;

        // toggle is_deleted on the board
        blocking(context.pool(), move |conn| {
            Board::update_deleted(conn, path.board_id.clone(), new_is_deleted)
        })
        .await??;

        let board_view = blocking(context.pool(), move |conn| {
            BoardView::read(conn, path.board_id.clone(), None)
        })
        .await??;

        Ok(BoardResponse { board_view })
    }
}