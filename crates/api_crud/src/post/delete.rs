use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{DeletePost, PostIdPath, PostResponse},
    utils::{
        blocking, check_board_ban, check_board_deleted_or_removed, check_user_valid,
        get_user_view_from_jwt,
    },
};
use tinyboards_db::{models::post::post::Post, traits::Crud};
use tinyboards_db_views::structs::PostView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for DeletePost {
    type Response = PostResponse;
    type Route = PostIdPath;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &DeletePost = &self;
        let user_view = get_user_view_from_jwt(auth, context.pool(), context.master_key()).await?;

        let post_id = path.post_id;
        let orig_post = blocking(context.pool(), move |conn| {
            Post::read(conn, post_id).map_err(|e| {
                eprintln!("ERROR: {}", e);
                TinyBoardsError::err_500()
            })
        })
        .await??;

        if orig_post.deleted == data.deleted {
            return Err(TinyBoardsError::from_string(
                "couldn't delete post again",
                500,
            ));
        }

        check_board_ban(user_view.user.id, orig_post.board_id, context.pool()).await?;

        check_board_deleted_or_removed(orig_post.board_id, context.pool()).await?;

        check_user_valid(
            user_view.user.banned,
            user_view.user.expires,
            user_view.user.deleted,
        )?;

        if !Post::is_post_creator(user_view.user.id, orig_post.creator_id) {
            return Err(TinyBoardsError::from_string("post edit not allowed", 405));
        }

        let post_id = path.post_id;
        let deleted = data.deleted;

        blocking(context.pool(), move |conn| {
            Post::update_deleted(conn, post_id, deleted).map_err(|_e| TinyBoardsError::err_500())
        })
        .await??;

        // grab the post view here for the response
        let post_view = blocking(context.pool(), move |conn| {
            PostView::read(conn, post_id, Some(user_view.user.id))
                .map_err(|_e| TinyBoardsError::from_string("could not find post", 404))
        })
        .await??;

        Ok(PostResponse { post_view })
    }
}
