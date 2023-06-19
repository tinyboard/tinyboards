use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    board::{RemoveBoard, BoardResponse},
    data::TinyBoardsContext,
    utils::{require_user}, build_response::{build_board_response},
};
use tinyboards_db::{
    models::{moderator::mod_actions::{ModRemoveBoard, ModRemoveBoardForm}, board::boards::Board},
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for RemoveBoard {
    type Response = BoardResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &RemoveBoard = &self;
        let orig_board = Board::read(context.pool(), data.board_id).await?;

        // require admin (only admin may remove a board)
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        let board_id = orig_board.id;
        let removed = data.removed;
        let updated_board = Board::update_removed(context.pool(), board_id, removed).await?;

        // mod log
        let form = ModRemoveBoardForm {
            mod_person_id: view.person.id,
            board_id: updated_board.id,
            removed: Some(Some(removed)),
            reason: Some(data.reason.clone()),
        };
        ModRemoveBoard::create(context.pool(), &form).await?;

        Ok(build_board_response(context, view, updated_board.id).await?)
    }
}