use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{EditPost, PostIdPath, PostResponse},
    utils::{
        check_board_deleted_or_removed, check_post_deleted_removed_or_locked,
        require_user,
    },
};
use tinyboards_db::{
    models::post::posts::{Post, PostForm},
    traits::Crud,
    utils::naive_now,
};
use tinyboards_db_views::structs::PostView;
use tinyboards_utils::{error::TinyBoardsError, parser::parse_markdown, utils::custom_body_parsing};

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
        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .not_banned()
            .unwrap()?;

        let post_id = path.post_id;
        let orig_post = PostView::read(context.pool(), post_id, None).await?;

        check_board_deleted_or_removed(orig_post.board.id, context.pool()).await?;

        check_post_deleted_removed_or_locked(orig_post.post.id, context.pool()).await?;

        if user.id != orig_post.post.creator_id {
            return Err(TinyBoardsError::from_message(403, "post edit not allowed"));
        }

        let body = Some(data.body.clone());
        // we need to re-parse the markdown here
        let mut body_html = parse_markdown(&body.clone().unwrap().as_str());
        body_html = Some(custom_body_parsing(&body_html.unwrap_or_default(), context.settings()));
        
        let post_id = path.post_id;
        // grabbing the current timestamp for the update
        let updated = Some(naive_now());

        let form = PostForm {
            body,
            body_html,
            updated,
            ..PostForm::default()
        };

        Post::update(context.pool(), post_id, &form)
            .await
            .map_err(|_| TinyBoardsError::from_message(500, "could not update post"))?;

        // parse post mentions here
        // send post notifications here (to mentioned users)

        let post_view = PostView::read(context.pool(), post_id, Some(orig_post.post.creator_id))
            .await
            .map_err(|_| TinyBoardsError::from_message(500, "could not find updated post"))?;

        Ok(PostResponse { post_view })
    }
}
