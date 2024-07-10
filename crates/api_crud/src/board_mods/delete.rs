use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    board::{BoardIdPersonIdPath, BoardModResponse, RemoveBoardMod},
    data::TinyBoardsContext,
    utils::{require_user, send_system_message},
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
impl<'des> PerformCrud<'des> for RemoveBoardMod {
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
        if board_id == 1 {
            return Err(TinyBoardsError::from_message(400, "Mods cannot be removed from the default board directly. Remove the user's admin, and their mod will be removed automatically."));
        }
        let board = Board::read(context.pool(), board_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to read board."))?;

        let mod_to_remove =
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
                Some(mod_to_remove.rank),
            )
            .await
            .unwrap()?;

        // if not removing self, send notification
        if v.person.id != person_id {
            let msg = send_system_message(
                context.pool(),
                Some(mod_to_remove.person_id),
                None,
                if mod_to_remove.invite_accepted {
                    format!(
                        "You have been removed as a moderator from [+{}](/+{}).",
                        board.name, board.name
                    )
                } else {
                    format!(
                        "Your moderator invite to [+{}](/+{}) has been revoked.",
                        board.name, board.name
                    )
                },
            )
            .await;

            if let Err(e) = msg {
                eprintln!(
                    "Sending mod removal notification failed with error: {:#?}",
                    e
                );
            }
        }

        mod_to_remove
            .remove(context.pool())
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Removing mod failed."))?;

        let moderators = BoardModeratorView::for_board(context.pool(), board_id).await?;

        Ok(BoardModResponse { moderators })
    }
}
