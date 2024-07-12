use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::board::BoardIdPath;
use tinyboards_api_common::{
    board::{BoardModResponse, InviteBoardMod},
    data::TinyBoardsContext,
    utils::{require_user, send_system_message},
};
use tinyboards_db::models::board::boards::Board;
use tinyboards_db::{
    models::board::board_mods::{BoardModerator, BoardModeratorForm, ModPerms},
    traits::Crud,
};
use tinyboards_db_views::structs::{BoardModeratorView, PersonView};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for InviteBoardMod {
    type Response = BoardModResponse;
    type Route = BoardIdPath;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        BoardIdPath { board_id }: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &InviteBoardMod = &self;
        //let board_id = data.board_id;

        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(context.pool(), board_id, ModPerms::Full, None)
            .await
            .unwrap()?;

        let permissions = data.permissions;

        let username = data.username.as_str();
        let target_person_view = PersonView::read_from_name(context.pool(), username)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 404, "Target user does not exist.")
            })?;

        let person_id = target_person_view.person.id;

        if target_person_view.person.is_banned {
            return Err(TinyBoardsError::from_message(
                403,
                format!("@{} is banned.", username).as_str(),
            ));
        }
        let board = Board::read(context.pool(), board_id).await?;

        if person_id == view.person.id {
            return Err(TinyBoardsError::from_message(
                400,
                "You cannot invite yourself. Why would you? Come on, man...",
            ));
        }

        let form = BoardModeratorForm {
            board_id: Some(board_id),
            person_id: Some(person_id),
            invite_accepted: Some(false),
            permissions: Some(permissions),
            ..BoardModeratorForm::default()
        };

        BoardModerator::create(context.pool(), &form).await?;

        let msg = send_system_message(
            context.pool(),
            Some(person_id),
            None,
            format!("You have been invited to become **a moderator** of [+{}](/+{}).\n\nGo to the [moderators page of +{}](/+{}/mod/mods) to accept or decline this invite.", board.name, board.name, board.name, board.name)
        ).await;

        // message failed to send - log the error to console, but save the mod invite anyways
        if let Err(e) = msg {
            eprintln!("Sending notification about moderator invite to @{} for board +{} failed with error: {:#?}", view.person.name, board.name, e);
        };

        let moderators = BoardModeratorView::for_board(context.pool(), board_id).await?;

        Ok(BoardModResponse { moderators })
    }
}
