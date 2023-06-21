use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
  board::{BoardResponse, SubscribeToBoard},
  data::TinyBoardsContext,
  utils::{check_board_deleted_or_removed, require_user, check_board_ban},
};
use tinyboards_db::{
  models::{
    apub::actor_language::BoardLanguage,
    board::boards::{Board},
    board::board_subscriber::{BoardSubscriber, BoardSubscriberForm},
  },
  traits::{Crud, Subscribeable},
};
use tinyboards_db_views::structs::BoardView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for SubscribeToBoard {
    type Response = BoardResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<BoardResponse, TinyBoardsError> {
        let data: &SubscribeToBoard = &self;
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;
        let board_id = data.board_id;
        let board = Board::read(context.pool(), board_id).await?;
        let board_subscriber_form = BoardSubscriberForm {
            board_id: data.board_id,
            person_id: view.person.id,
            pending: Some(false),
        };

        if board.local && data.subscribe {
            check_board_ban(view.person.id, board.id, context.pool()).await?;
            check_board_deleted_or_removed(board.id, context.pool()).await?;
            BoardSubscriber::subscribe(context.pool(), &board_subscriber_form).await?;
        }
        if !data.subscribe {
            BoardSubscriber::unsubscribe(context.pool(), &board_subscriber_form).await?;
        }

        let board_id = data.board_id;
        let person_id = view.person.id;
        let board_view = 
            BoardView::read(context.pool(), board_id, Some(person_id), None).await?;
        let discussion_languages = BoardLanguage::read(context.pool(), board_id).await?;

        Ok(BoardResponse { board_view, discussion_languages })
    }
}