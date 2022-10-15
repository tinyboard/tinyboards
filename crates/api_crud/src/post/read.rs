use crate::PerformCrud;
use actix_web::web::Data;
use porpl_api_common::{
    post::{GetPost, GetPostResponse, PostIdPath},
    utils::{blocking, get_user_view_from_jwt}, data::PorplContext,
};
// use porpl_db::{
//     //aggregates::structs::PostAggregates,
//     //models::comment::comment::Comment,
//     traits::{Crud}
// };
use porpl_utils::PorplError;

use porpl_db_views::structs::{
    PostView,
    BoardView,
    BoardModeratorView,
};

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for GetPost {
    type Response = GetPostResponse;
    type Route = PostIdPath;

    async fn perform(
        self,
        context: &Data<PorplContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<GetPostResponse, PorplError> {

        let _data = self;
        // check private instancce

        let u_view = 
            get_user_view_from_jwt(auth.unwrap(), context.pool(), context.master_key()).await?;

        let uid = u_view.user.id;

        let post_id = path.post_id;

        let post_view = blocking(context.pool(), move |conn| {
            PostView::read(conn, post_id, Some(uid))
        })
        .await?
        .map_err(|e| {
            eprintln!("ERROR: {}", e);
            PorplError::err_500()
        })?;

        let _post_id = post_view.post.id;

        // mark read here 

        let board_id = post_view.board.id;
        let board_view = blocking(context.pool(), move |conn| {
            BoardView::read(conn, board_id, Some(uid))
        })
        .await?
        .map_err(|e| {
            eprintln!("ERROR: {}", e);
            PorplError::err_500()
        })?;

        // blank out deleted or removed info here

        let moderators = blocking(context.pool(), move |conn| {
            BoardModeratorView::for_board(conn, board_id)
                .map_err(|e| {
                    eprintln!("ERROR: {}", e);
                    PorplError::err_500()
                })
        })
        .await??;

        Ok(GetPostResponse { 
            post_view, 
            board_view, 
            moderators
         })

    }
}