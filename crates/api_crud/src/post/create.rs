use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{SubmitPost, PostResponse},
    utils::{
        blocking, check_board_ban, check_board_deleted_or_removed, check_user_valid,
        get_user_view_from_jwt,
    },
};
use tinyboards_db::{models::post::{post::{Post, PostForm}, post_vote::{PostVoteForm, PostVote}}, traits::Voteable};
use tinyboards_utils::{parser::parse_markdown, TinyBoardsError};
use tinyboards_db_views::structs::PostView;

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

        let user_view =
            get_user_view_from_jwt(auth.unwrap(), context.pool(), context.master_key()).await?;

        // check to see if user is banned from board
        check_board_ban(
            user_view.user.id,
            data.board_id.unwrap_or(1),
            context.pool(),
        )
        .await?;

        // check to see if board is removed or deleted
        check_board_deleted_or_removed(data.board_id.unwrap_or(1), context.pool()).await?;

        // check to see if user is valid (not banned or deleted)
        check_user_valid(
            user_view.user.banned,
            user_view.user.expires,
            user_view.user.deleted,
        )?;

        let body_html = match data.body {
            Some(ref body) => parse_markdown(body),
            None => None,
        };

        let post_form = PostForm {
            title: data.title,
            type_: data.type_,
            url: data.url,
            body: data.body,
            body_html: body_html,
            creator_id: user_view.user.id,
            board_id: data.board_id.unwrap_or(1),
            nsfw: Some(data.nsfw),
            ..PostForm::default()
        };

        let published_post =
            blocking(context.pool(), move |conn| Post::submit(conn, post_form)
                .map_err(|_e| TinyBoardsError::from_string("could not submit post", 500)))
                .await??;

  

        // auto upvote own post
        let post_vote = PostVoteForm {
            post_id: published_post.id,
            user_id: user_view.user.id,
            score: 1,
        };

        blocking(context.pool(), move |conn| {
            PostVote::vote(conn, &post_vote)
        })
        .await??;

         

        let post_view = blocking(context.pool(), move |conn| {
            PostView::read(conn, published_post.id, Some(user_view.user.id))
                .map_err(|_e| TinyBoardsError::from_string("could not find newly published post", 404))
        })
        .await??;

        Ok( PostResponse { post_view } )
    }
}
