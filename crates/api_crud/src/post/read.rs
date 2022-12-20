use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{GetPost, GetPostResponse, PostIdPath},
    utils::{blocking, check_private_instance, load_user_opt},
};

use tinyboards_utils::TinyBoardsError;

use tinyboards_db_views::{
    structs::{BoardModeratorView, BoardView, PostView},
    DeleteableOrRemoveable,
};

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for GetPost {
    type Response = GetPostResponse;
    type Route = PostIdPath;

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<GetPostResponse, TinyBoardsError> {
        let _data = self;

        let user = load_user_opt(context.pool(), context.master_key(), auth).await?;

        // check to see if instance is set to private before listing post
        check_private_instance(&user, context.pool()).await?;

        let user_id = match user {
            Some(ref user) => Some(user.id),
            None => None,
        };

        let post_id = path.post_id;

        let mut post_view = blocking(context.pool(), move |conn| {
            PostView::read(conn, post_id, user_id)
        })
        .await??;

        if post_view.post.is_removed || post_view.post.is_deleted {
            post_view.hide_if_removed_or_deleted(user.as_ref());
        }

        let _post_id = post_view.post.id;

        // mark read here

        let board_id = post_view.board.id;
        let board_view = blocking(context.pool(), move |conn| {
            BoardView::read(conn, board_id, user_id)
        })
        .await??;

        // blank out deleted or removed info here

        let moderators = blocking(context.pool(), move |conn| {
            BoardModeratorView::for_board(conn, board_id)
        })
        .await??;

        Ok(GetPostResponse {
            post_view,
            board_view,
            moderators,
        })
    }
}
