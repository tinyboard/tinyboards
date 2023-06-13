use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    board::{CreateBoard, BoardResponse},
    utils::{
        require_user,
    },
};
use tinyboards_db::{
    models::{
        board::boards::{Board, BoardForm},
    },
    traits::Crud,
};
use tinyboards_db_views::structs::BoardView;
use tinyboards_utils::{
    parser::parse_markdown,
    TinyBoardsError,
};

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for CreateBoard {
    type Response = BoardResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<BoardResponse, TinyBoardsError> {
        let data: &CreateBoard = &self;

        let name = data.name.clone();
        let title = data.title.clone();
        let mut description = data.description.clone();


        // board creation restricted to admins (may provide other options in the future)
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        if let Some(desc) = description {
            description = parse_markdown(&desc);
        }

        let form = BoardForm {
            name: Some(name),
            title: Some(title),
            description: Some(description),
            creator_id: Some(view.person.id),
            ..BoardForm::default()
        };

        // create the board
        let board = Board::create(context.pool(), &form).await?;

        let board_view = BoardView::read(context.pool(), board.id, None).await?;

        Ok(BoardResponse { board_view })
    }
}