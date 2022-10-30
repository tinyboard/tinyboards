use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{EditPost, PostIdPath, PostResponse},
    utils::{
        blocking, check_board_ban, check_board_deleted_or_removed,
        check_post_deleted_removed_or_locked, get_user_view_from_jwt,
    },
};
use tinyboards_db::{
    models::post::post::{Post, PostForm},
    traits::Crud,
};
use tinyboards_db_views::structs::PostView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for EditPost {
    type Response = PostResponse;
    type Route = PostIdPath;

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<PostResponse, TinyBoardsError> {
        let data: &EditPost = &self;
        let user_view =
            get_user_view_from_jwt(auth.unwrap_or(""), context.pool(), context.master_key())
                .await?;

        let post_id = path.post_id;
        let orig_post = blocking(context.pool(), move |conn| {
            PostView::read(conn, post_id, None)
                .map_err(|_e| TinyBoardsError::from_string("could not find original post", 404))
        })
        .await??;

        check_board_ban(user_view.user.id, orig_post.board.id, context.pool()).await?;

        check_board_deleted_or_removed(orig_post.board.id, context.pool()).await?;

        check_post_deleted_removed_or_locked(orig_post.post.id, context.pool()).await?;

        if user_view.user.id != orig_post.post.creator_id {
            return Err(TinyBoardsError::from_string("post edit not allowed", 405));
        }

        let body = data.body.clone();
        let body_html = data.body_html.clone();
        let post_id = path.post_id;

        let form = PostForm {
            creator_id: orig_post.post.creator_id,
            body,
            body_html,
            ..PostForm::default()
        };

        blocking(context.pool(), move |conn| {
            Post::update(conn, post_id, &form)
                .map_err(|_e| TinyBoardsError::from_string("could not update post", 500))
        })
        .await??;

        // parse post mentions here
        // send post notifications here (to mentioned users)

        let post_view = blocking(context.pool(), move |conn| {
            PostView::read(conn, post_id, Some(orig_post.post.creator_id))
                .map_err(|_e| TinyBoardsError::from_string("could not find updated post", 404))
        })
        .await??;

        Ok(PostResponse { post_view })
    }
}
