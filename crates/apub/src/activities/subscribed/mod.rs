use crate::{
    objects::board::ApubBoard,
    protocol::activities::subscribed::{subscribe::Subscribe, undo_subscribe::UndoSubscribe},
    SendActivity,
};
use tinyboards_federation::config::Data;
use tinyboards_api_common::{
    board::{BoardResponse, SubscribeToBoard},
    data::TinyBoardsContext,
    utils::require_user,
};
use tinyboards_db::{models::board::boards::Board, traits::Crud};
use tinyboards_utils::error::TinyBoardsError;

pub mod accept;
pub mod subscribe;
pub mod undo_subscribe;

#[async_trait::async_trait]
impl SendActivity for SubscribeToBoard {
    type Response = BoardResponse;

    async fn send_activity(
        request: &Self,
        _response: &Self::Response,
        context: &Data<TinyBoardsContext>,
        auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        let view = require_user(context.pool(), context.master_key(), auth).await.unwrap()?;
        let person = view.person.clone().into();
        let board: ApubBoard = Board::read(context.pool(), request.board_id)
            .await?
            .into();
        if board.local {
            Ok(())
        } else if request.follow {
            Subscribe::send(&person, &board, context).await
        } else {
            UndoSubscribe::send(&person, &board, context).await
        }
    }
}