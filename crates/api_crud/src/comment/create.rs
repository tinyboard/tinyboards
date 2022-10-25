use crate::PerformCrud;
use actix_web::web;
use porpl_api_common::{
    comment::CreateComment,
    data::PorplContext,
    utils::{
        blocking, check_board_ban, check_board_deleted_or_removed,
        check_post_deleted_removed_or_locked, check_user_valid, get_user_view_from_jwt,
    },
};
use porpl_db::{
    models::{
        comment::{
            comment::{Comment, CommentForm},
            comment_vote::{CommentVote, CommentVoteForm},
        },
        post::post::Post,
    },
    traits::{Crud, Voteable},
};
use porpl_db_views::structs::CommentView;
use porpl_utils::{parser::parse_markdown, PorplError};

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for CreateComment {
    type Response = CommentView;
    type Route = ();

    async fn perform(
        self,
        context: &web::Data<PorplContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, PorplError> {
        let data = self;

        let user_view =
            get_user_view_from_jwt(auth.unwrap(), context.pool(), context.master_key()).await?;

        let post = blocking(context.pool(), move |conn| {
            Post::read(conn, data.post_id).map_err(|_| PorplError::err_500())
        })
        .await??;

        // checks to see if the board even exists in the first place
        check_board_deleted_or_removed(post.board_id, context.pool()).await?;
        // checks the board to see if user is banned
        check_board_ban(user_view.user.id, post.board_id, context.pool()).await?;

        // checks to see if the post was deleted, removed, or locked
        check_post_deleted_removed_or_locked(post.id, context.pool()).await?;

        // checks the user to see if the user is site banned (is valid or not)
        check_user_valid(
            user_view.user.banned,
            user_view.user.expires,
            user_view.user.deleted,
        )?;

        // check if parent comment exists
        // TODO: check if post's op is blocking the user (?)
        if blocking(context.pool(), move |conn| {
            Post::check_if_exists(conn, data.post_id)
        })
        .await??
        .is_none()
        {
            return Err(PorplError::from_string("Invalid post ID", 404));
        }

        let mut level = 1;
        // check if parent comment exists, if provided
        // TODO: check if comment's author is blocking the user (?)
        if let Some(cid) = data.parent_id {
            let parent_comment =
                blocking(context.pool(), move |conn| Comment::get_by_id(conn, cid)).await??;
            if parent_comment.is_none() {
                return Err(PorplError::from_string("Invalid parent comment ID", 404));
            }

            // we can unwrap safely, because the above check made sure to abort if the comment is None
            // abort if the comment the user is replying to doesn't belong to the specified post - may be useful later
            let parent_comment = parent_comment.unwrap();
            if parent_comment.post_id != data.post_id {
                return Err(PorplError::from_string(
                    "What a bad request! Now you have a good reason to be ashamed of yourself.",
                    400,
                ));
            }

            level = parent_comment.level + 1;
        }

        let body_html = parse_markdown(&data.body);

        // TODO: scrape comment text for @mentions and send notifs
        let new_comment = CommentForm {
            creator_id: user_view.user.id,
            body: Some(data.body),
            body_html,
            post_id: data.post_id,
            parent_id: data.parent_id,
            level: Some(level),
            ..CommentForm::default()
        };

        let new_comment = blocking(context.pool(), move |conn| {
            Comment::submit(conn, new_comment)
        })
        .await??;

        // auto upvote own comment
        let comment_vote = CommentVoteForm {
            user_id: user_view.user.id,
            comment_id: new_comment.id,
            score: 1,
        };

        blocking(context.pool(), move |conn| {
            CommentVote::vote(conn, &comment_vote)
        })
        .await??;

        let new_comment = blocking(context.pool(), move |conn| {
            CommentView::read(conn, new_comment.id, Some(user_view.user.id))
        })
        .await?
        .map_err(|_| PorplError::err_500())?;

        Ok(new_comment)
    }
}
