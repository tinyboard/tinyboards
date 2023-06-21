use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{PostResponse, SubmitPost},
    utils::{check_board_deleted_or_removed, require_user, generate_local_apub_endpoint, EndpointType}, build_response::build_post_response,
};
use tinyboards_db::{
    models::{post::{
        post_votes::{PostVote, PostVoteForm},
        posts::{Post, PostForm},
    }, apub::actor_language::BoardLanguage},
    traits::{Voteable, Crud}, impls::apub::actor_language::default_post_language,
};
use tinyboards_utils::{parser::parse_markdown, TinyBoardsError, utils::custom_body_parsing};

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for SubmitPost {
    type Response = PostResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<PostResponse, TinyBoardsError> {
        let data: SubmitPost = self;
        let board_id = data.board_id.unwrap_or(1);

        let view = require_user(context.pool(), context.master_key(), auth)
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

        let language_id = match data.language_id {
            Some(lid) => Some(lid),
            None => {
                default_post_language(context.pool(), board_id.clone(), view.local_user.id).await?
            }
        };

        BoardLanguage::is_allowed_board_language(context.pool(), language_id, board_id.clone()).await?;
        
        let post_form = PostForm {
            title: Some(data.title),
            type_: data.type_,
            url: data.url,
            image: data.image,
            body: Some(body), // once told me, the world was gonna roll me
            body_html: body_html,
            creator_id: Some(view.person.id),
            board_id: Some(board_id),
            is_nsfw: Some(data.is_nsfw),
            language_id: language_id.clone(),
            ..PostForm::default()
        };

        let published_post = Post::submit(context.pool(), post_form).await?;
        
        // apub id add
        let protocol_and_hostname = context.settings().get_protocol_and_hostname();
        let apub_id = generate_local_apub_endpoint(
            EndpointType::Post, 
            &published_post.id.clone().to_string(), 
            &protocol_and_hostname,
        )?;
        let update_form = PostForm {
            ap_id: Some(apub_id),
            ..PostForm::default()
        };
        let updated_post = Post::update(
            context.pool(), 
            published_post.id.clone(), 
            &update_form
        ).await?;

        // auto upvote own post
        let post_vote = PostVoteForm {
            post_id: updated_post.id,
            person_id: view.person.id,
            score: 1,
        };

        PostVote::vote(context.pool(), &post_vote).await?;

        // TODO:
        // logic to mark post as read for the poster

        // web mention logic

        Ok(build_post_response(context, board_id, view.person.id, published_post.id).await?)
    }
}
