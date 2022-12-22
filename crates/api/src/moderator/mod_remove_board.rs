use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    moderator::{BanBoard, ModActionResponse},
    utils::{blocking, require_user},
};
use tinyboards_db::{
    models::board::boards::Board,
    models::moderator::mod_actions::{ModRemoveBoard, ModRemoveBoardForm},
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for BanBoard {
    type Response = ModActionResponse<ModRemoveBoard>;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &BanBoard = &self;

        let board_id = data.board_id;
        let reason = data.reason.clone();
        let banned = data.banned;

        if board_id == 1 {
            return Err(TinyBoardsError::from_message(
                "you can't ban the default board",
            ));
        }

        // require a mod/admin for this action
        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(board_id.clone(), context.pool())
            .await
            .unwrap()?;

        // update the board in the database
        blocking(context.pool(), move |conn| {
            Board::update_banned(conn, board_id.clone(), banned)
        })
        .await??;

        // form for submitting remove action to mod log
        let remove_board_form = ModRemoveBoardForm {
            mod_user_id: user.id,
            board_id: board_id.clone(),
            reason: Some(reason),
            removed: Some(Some(banned)),
        };

        // submit mod action to the mod log
        let mod_action = blocking(context.pool(), move |conn| {
            ModRemoveBoard::create(conn, &remove_board_form)
        })
        .await??;

        Ok(ModActionResponse { mod_action })
    }
}
