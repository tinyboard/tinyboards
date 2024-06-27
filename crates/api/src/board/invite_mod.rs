use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::board::BoardIdPath;
use tinyboards_api_common::{
    board::{BoardModResponse, InviteBoardMod},
    data::TinyBoardsContext,
    utils::require_user,
};
use tinyboards_db::{
    models::board::board_mods::{BoardModerator, BoardModeratorForm, ModPerms},
    traits::Crud,
};
use tinyboards_db_views::structs::BoardModeratorView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for InviteBoardMod {
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
            .require_board_mod(context.pool(), board_id, ModPerms::Full)
            .await
            .unwrap()?;

        let person_id = data.person_id;
        let permissions = data.permissions;

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

        let moderators = BoardModeratorView::for_board(context.pool(), board_id).await?;

        Ok(BoardModResponse { moderators })
    }
}
