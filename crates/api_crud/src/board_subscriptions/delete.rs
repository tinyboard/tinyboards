use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    board::{BoardIdPath, BoardResponse, UnsubFromBoard},
    data::TinyBoardsContext,
    utils::require_user,
};
use tinyboards_db::{
    models::{
        apub::actor_language::BoardLanguage,
        board::board_subscriber::{BoardSubscriber, BoardSubscriberForm},
        board::boards::Board,
    },
    traits::{Crud, Subscribeable},
};
use tinyboards_db_views::structs::BoardView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for UnsubFromBoard {
    type Response = BoardResponse;
    type Route = BoardIdPath;

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        BoardIdPath { board_id }: Self::Route,
        auth: Option<&str>,
    ) -> Result<BoardResponse, TinyBoardsError> {
        //let data: &UnsubFromBoard = &self;
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        let _ = Board::read(context.pool(), board_id).await?;

        let board_subscriber_form = BoardSubscriberForm {
            board_id,
            person_id: view.person.id,
            pending: Some(false),
        };

        BoardSubscriber::unsubscribe(context.pool(), &board_subscriber_form).await?;

        //let board_id = data.board_id;
        let person_id = view.person.id;
        let board_view = BoardView::read(context.pool(), board_id, Some(person_id), None).await?;
        let discussion_languages = BoardLanguage::read(context.pool(), board_id).await?;

        Ok(BoardResponse {
            board_view,
            discussion_languages,
        })
    }
}
