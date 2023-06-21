use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    board::{BlockBoard, BlockBoardResponse},
    data::TinyBoardsContext,
    utils::require_user,
};
use tinyboards_db::{
    models::{
        board::board_subscriber::{BoardSubscriber, BoardSubscriberForm},
        board::board_block::{BoardBlock, BoardBlockForm},
    },
    traits::{Blockable, Subscribeable},
};
use tinyboards_db_views::structs::BoardView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for BlockBoard {
    type Response = BlockBoardResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<BlockBoardResponse, TinyBoardsError> {
        let data: &BlockBoard = &self;

        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        let board_id = data.board_id;
        let person_id = view.person.id;
        let board_block_form = BoardBlockForm {
            person_id,
            board_id
        };

        if data.block {
            BoardBlock::block(context.pool(), &board_block_form)
                .await?;

            // also unsubscribe from the board and send a federated unsubscribe
            let board_subscriber_form = BoardSubscriberForm {
                board_id: data.board_id,
                person_id,
                pending: Some(false),
            };
            BoardSubscriber::unsubscribe(context.pool(), &board_subscriber_form)
                .await
                .ok();

        } else {
            BoardBlock::unblock(context.pool(), &board_block_form)
                .await?;
        }

        let board_view = BoardView::read(context.pool(), board_id, Some(person_id), None).await?;

        Ok(BlockBoardResponse { board_view, blocked: data.block })
    }
}