use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    board::{DeleteBoard, BoardIdPath, BoardResponse},
    utils::{
        require_user,
    }, build_response::build_board_response,
};

use tinyboards_db::{
    models::{
        board::boards::Board,
    },
    traits::Crud, 
};
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
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        let orig_board = Board::read(context.pool(), path.board_id.clone()).await?;

        // negate the deleted status from the board as it is currently in the database to get new deleted status
        let new_is_deleted = !orig_board.is_deleted;

        // toggle is_deleted on the board
        let deleted_board = Board::update_deleted(context.pool(), path.board_id.clone(), new_is_deleted).await?;

        Ok(build_board_response(context, view, deleted_board.id).await?)
    }
}