use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    board::{BoardResponse, HideBoard},
    build_response::build_board_response,
    data::TinyBoardsContext,
    utils::require_user,
};
use tinyboards_db::{
    models::{
        board::boards::Board,
        moderator::mod_actions::{ModHideBoard, ModHideBoardForm},
        person::local_user::AdminPerms,
    },
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for HideBoard {
    type Response = BoardResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<BoardResponse, TinyBoardsError> {
        let data: &HideBoard = &self;

        // verify that the requester is an admin
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin(AdminPerms::Boards)
            .unwrap()?;

        Board::update_hidden(context.pool(), data.board_id, data.hidden).await?;

        let mod_hide_board_form = ModHideBoardForm {
            board_id: Some(data.board_id),
            mod_person_id: Some(view.person.id),
            reason: data.reason.clone(),
            hidden: Some(data.hidden),
        };

        ModHideBoard::create(context.pool(), &mod_hide_board_form).await?;

        Ok(build_board_response(context, view, data.board_id).await?)
    }
}
