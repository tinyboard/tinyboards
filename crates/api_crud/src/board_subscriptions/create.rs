use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    board::{BoardResponse, SubscribeToBoard},
    data::TinyBoardsContext,
    utils::{check_board_ban, require_user},
};
use tinyboards_db::{
    models::{
        apub::actor_language::BoardLanguage,
        board::board_subscriber::{BoardSubscriber, BoardSubscriberForm},
        board::boards::Board,
    },
    traits::Subscribeable,
};
use tinyboards_db_views::structs::BoardView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for SubscribeToBoard {
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
        let board_name = data.board_name.as_str();
        let board = Board::get_by_name(context.pool(), board_name).await?;

        let board_subscriber_form = BoardSubscriberForm {
            board_id: board.id,
            person_id: view.person.id,
            pending: Some(false),
        };

        if board.local {
            check_board_ban(view.person.id, board.id, context.pool()).await?;
            BoardSubscriber::subscribe(context.pool(), &board_subscriber_form).await?;
        }
        /*if !data.subscribe {
            BoardSubscriber::unsubscribe(context.pool(), &board_subscriber_form).await?;
        }*/

        let board_id = board.id;
        let person_id = view.person.id;
        let board_view = BoardView::read(context.pool(), board_id, Some(person_id), None).await?;
        let discussion_languages = BoardLanguage::read(context.pool(), board_id).await?;

        Ok(BoardResponse {
            board_view,
            discussion_languages,
        })
    }
}
