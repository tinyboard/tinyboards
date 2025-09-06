use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    board::{GetBoard, GetBoardResponse},
    data::TinyBoardsContext,
    utils::{check_private_instance, load_user_opt},
};
use tinyboards_db_views::structs::{BoardModeratorView, BoardView};
use tinyboards_utils::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetBoard {
    type Response = GetBoardResponse;
    type Route = ();

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _path: Self::Route,
        auth: Option<&str>,
    ) -> Result<GetBoardResponse, TinyBoardsError> {
        let data = self;
        let v = load_user_opt(context.pool(), context.master_key(), auth).await?;
        
        // check to see if instance is set to private before listing board
        check_private_instance(&v, context.pool()).await?;

        let person_id = v.as_ref().map(|u| u.person.id);
        
        // Use board id from the request data
        let board_id = data.id.ok_or_else(|| {
            TinyBoardsError::from_message(400, "board id is required")
        })?;
        
        let board_view = BoardView::read(context.pool(), board_id, person_id, None).await?;
        let moderators = BoardModeratorView::for_board(context.pool(), board_id).await?;

        Ok(GetBoardResponse {
            board_view,
            site: None,
            moderators,
            discussion_languages: vec![],
        })
    }
}