use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    admin::{PurgeBoard, PurgeItemResponse},
    data::TinyBoardsContext,
    utils::{purge_local_image_posts_for_board, require_user},
};
use tinyboards_db::{
    models::{
        board::boards::Board,
        moderator::admin_actions::{AdminPurgeBoard, AdminPurgeBoardForm},
    },
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for PurgeBoard {
    type Response = PurgeItemResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &PurgeBoard = &self;

        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        let target_board_id = data.board_id;
        let reason = data.reason.clone();

        // let board = blocking(context.pool(), move |conn| {
        //     Board::read(conn, target_board_id)
        // })
        // .await??;

        // TODO - add in purging of board banner/icon

        // purge image posts for board
        purge_local_image_posts_for_board(
            target_board_id,
            context.pool(),
        )
        .await?;

        // delete board
        Board::delete(context.pool(), target_board_id).await?;

        let form = AdminPurgeBoardForm {
            admin_id: view.person.id,
            board_id: target_board_id,
            reason: Some(reason),
        };

        // submit mod log action
        AdminPurgeBoard::create(context.pool(), &form).await?;

        Ok(PurgeItemResponse { success: true })
    }
}
