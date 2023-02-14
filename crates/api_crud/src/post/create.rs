use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{PostResponse, SubmitPost},
    utils::{check_board_deleted_or_removed, require_user},
};
use tinyboards_db::{
    models::{post::{
        post_votes::{PostVote, PostVoteForm},
        posts::{Post, PostForm},
    }, site::stray_images::StrayImage},
    traits::Voteable,
};
use tinyboards_db_views::structs::PostView;
use tinyboards_utils::{parser::parse_markdown, TinyBoardsError, utils::custom_body_parsing};

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for SubmitPost {
    type Response = PostResponse;
    type Route = ();

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<PostResponse, TinyBoardsError> {
        let data: SubmitPost = self;
        let board_id = data.board_id.unwrap_or(1);

        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .not_banned()
            .not_banned_from_board(board_id, context.pool())
            .await
            .unwrap()?;

        // check to see if board is removed or deleted
        check_board_deleted_or_removed(data.board_id.unwrap_or(1), context.pool()).await?;

        let body = data.body.unwrap_or_default();
        let mut body_html = parse_markdown(&body.as_str());
        body_html = Some(custom_body_parsing(&body_html.unwrap_or_default(), context.settings()));
        
        let post_form = PostForm {
            title: Some(data.title),
            type_: data.type_,
            url: data.url,
            image: data.image,
            body: Some(body), // once told me, the world was gonna roll me
            body_html: body_html,
            creator_id: Some(user.id),
            board_id: Some(board_id),
            is_nsfw: Some(data.is_nsfw),
            ..PostForm::default()
        };

        let published_post = Post::submit(context.pool(), post_form).await?;

        // remove image url from the stray image deletion queue if it's actually posted
        if published_post.image.is_some() {
            StrayImage::remove_url_from_stray_images(context.pool(), published_post.image.unwrap()).await?;
        }

        // auto upvote own post
        let post_vote = PostVoteForm {
            post_id: published_post.id,
            user_id: user.id,
            score: 1,
        };

        PostVote::vote(context.pool(), &post_vote).await?;

        let post_view = PostView::read(context.pool(), published_post.id, Some(user.id)).await?;

        Ok(PostResponse { post_view })
    }
}
