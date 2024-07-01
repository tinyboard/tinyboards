use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::board::BoardIdPath;
use tinyboards_api_common::{
    board::{BoardResponse, ToggleBoardBan},
    build_response::build_board_response,
    data::TinyBoardsContext,
    utils::require_user,
};
use tinyboards_db::{
    models::{
        board::boards::Board,
        moderator::mod_actions::{ModRemoveBoard, ModRemoveBoardForm},
        person::local_user::AdminPerms,
    },
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for ToggleBoardBan {
    type Response = BoardResponse;
    type Route = BoardIdPath;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        BoardIdPath { board_id }: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &ToggleBoardBan = &self;
        let board = Board::read(context.pool(), board_id).await?;
        let reason = &data.reason;

        // require admin (only admin may remove a board)
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin(AdminPerms::Boards)
            .unwrap()?;

        //let board_id = orig_board.id;
        let ban = data.value;
        //let updated_board = Board::update_removed(context.pool(), board_id, removed).await?;

        let _ = if ban {
            board.ban(context.pool(), reason.as_ref()).await
        } else {
            board.unban(context.pool()).await
        }
        .map_err(|e| {
            TinyBoardsError::from_error_message(e, 500, "Updating board banned status failed.")
        })?;

        // mod log
        let form = ModRemoveBoardForm {
            mod_person_id: view.person.id,
            board_id: board.id,
            removed: Some(Some(ban)),
            reason: Some(data.reason.clone()),
        };
        ModRemoveBoard::create(context.pool(), &form).await?;

        Ok(build_board_response(context, view, board.id).await?)
    }
}
