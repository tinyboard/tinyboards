use crate::{
    objects::board::ApubBoard,
    protocol::activities::subscribed::{subscribe::Subscribe, undo_subscribe::UndoSubscribe},
    SendActivity,
};
use tinyboards_api_common::{
    board::{BoardIdPath, BoardResponse, SubscribeToBoard, UnsubFromBoard},
    data::TinyBoardsContext,
    utils::require_user,
};
use tinyboards_db::{models::board::boards::Board, traits::Crud};
use tinyboards_federation::config::Data;
use tinyboards_utils::error::TinyBoardsError;

pub mod accept;
pub mod subscribe;
pub mod undo_subscribe;

#[async_trait::async_trait]
impl SendActivity for SubscribeToBoard {
    type Response = BoardResponse;
    type Route = ();

    async fn send_activity(
        request: &Self,
        _response: &Self::Response,
        context: &Data<TinyBoardsContext>,
        _: &Self::Route,
        auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;
        let person = view.person.clone().into();
        let board: ApubBoard = Board::get_by_name(context.pool(), &request.board_name)
            .await?
            .into();
        if board.local {
            Ok(())
        } else {
            Subscribe::send(&person, &board, context).await
        } /*else {
              UndoSubscribe::send(&person, &board, context).await
          }*/
    }
}

#[async_trait::async_trait]
impl SendActivity for UnsubFromBoard {
    type Response = BoardResponse;
    type Route = BoardIdPath;

    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        context: &Data<TinyBoardsContext>,
        &BoardIdPath { board_id }: &Self::Route,
        auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;
        let person = view.person.clone().into();
        let board: ApubBoard = Board::read(context.pool(), board_id).await?.into();
        if !board.local {
            UndoSubscribe::send(&person, &board, context).await
        } else {
            Ok(())
        }
    }
}
