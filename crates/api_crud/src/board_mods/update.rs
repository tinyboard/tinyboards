use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    board::{BoardIdPersonIdPath, BoardModResponse, EditBoardMod},
    data::TinyBoardsContext,
    utils::require_user,
};
use tinyboards_db::{
    models::board::{
        board_mods::{BoardModerator, ModPerms},
        boards::Board,
    },
    traits::Crud,
};
use tinyboards_db_views::structs::BoardModeratorView;
use tinyboards_utils::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for EditBoardMod {
    type Response = BoardModResponse;
    type Route = BoardIdPersonIdPath;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        BoardIdPersonIdPath {
            board_id,
            person_id,
        }: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let mod_to_update =
            BoardModerator::get_by_person_id_for_board(context.pool(), person_id, board_id)
                .await
                .map_err(|e| {
                    TinyBoardsError::from_error_message(e, 400, "That user is not a board mod.")
                })?;

        let v = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(
                context.pool(),
                board_id,
                ModPerms::Full,
                Some(mod_to_update.rank),
            )
            .await
            .unwrap()?;

        mod_to_update
            .set_permissions(context.pool(), self.permissions)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Removing mod failed."))?;

        let moderators = BoardModeratorView::for_board(context.pool(), board_id).await?;

        Ok(BoardModResponse { moderators })
    }
}
