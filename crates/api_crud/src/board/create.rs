use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    board::{CreateBoard, CreateBoardResponse},
    utils::{
        blocking,
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
    type Response = CreateBoardResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<CreateBoardResponse, TinyBoardsError> {
        let data: &CreateBoard = &self;

        let name = data.name.clone();
        let title = data.title.clone();
        let mut description = data.description.clone();


        // board creation restricted to admins (may provide other options in the future)
        let user = require_user(context.pool(), context.master_key(), auth)
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
            creator_id: Some(user.id),
            ..BoardForm::default()
        };

        // create the board
        let board = blocking(context.pool(), move |conn| {
            Board::create(conn, &form)
        })
        .await??;

        let board_view = blocking(context.pool(), move |conn| {
            BoardView::read(conn, board.id, None)
        })
        .await??;

        Ok(CreateBoardResponse { board_view })
    }
}