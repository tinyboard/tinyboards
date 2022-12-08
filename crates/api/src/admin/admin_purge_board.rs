use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    admin::PurgeBoard,
    utils::{blocking, require_user, purge_image_posts_for_board}, 
    moderator::ModActionResponse, 
    data::TinyBoardsContext,
};
use tinyboards_db::{
    models::{
        moderator::admin_actions::{AdminPurgeBoard, AdminPurgeBoardForm},
        board::board::Board,
    },
    traits::Crud,
};
use tinyboards_utils::{
    error::TinyBoardsError,
};

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for PurgeBoard {
    type Response = ModActionResponse<AdminPurgeBoard>;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &PurgeBoard = &self;

        let user
            = require_user(context.pool(), context.master_key(), auth)
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
        purge_image_posts_for_board(
            target_board_id,
            context.pool(),
            context.settings(),
            context.client(),
        )
        .await?;


        // delete board
        blocking(context.pool(), move |conn| {
            Board::delete(conn, target_board_id)
        })
        .await??;

        let form = AdminPurgeBoardForm {
            admin_id: user.id,
            board_id: target_board_id,
            reason: Some(reason),
        };

        // submit mod log action
        let mod_action = blocking(context.pool(), move |conn| {
            AdminPurgeBoard::create(conn, &form)
        })
        .await??;

        Ok(ModActionResponse { mod_action })
    }
}