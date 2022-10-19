use crate::PerformCrud;
use actix_web::web;
use porpl_api_common::{
    comment::CreateComment,
    data::PorplContext,
    utils::{blocking, require_user},
};
use porpl_db::{
    models::{
        comment::{
            comment::{Comment, CommentForm},
            comment_like::{CommentLike, CommentLikeForm},
        },
        post::post::Post,
    },
    traits::Likeable,
};
use porpl_utils::PorplError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for CreateComment {
    type Response = Comment;
    type Route = ();

    async fn perform(
        self,
        context: &web::Data<PorplContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, PorplError> {
        let data = self;

        let user = require_user(context.pool(), context.master_key(), auth).await?;

        // TODO: check for board ban

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

        // check if parent comment exists, if provided
        // TODO: check if comment's author is blocking the user (?)
        if let Some(cid) = data.parent_id {
            if blocking(context.pool(), move |conn| {
                Comment::check_if_exists(conn, cid)
            })
            .await??
            .is_none()
            {
                return Err(PorplError::from_string("Invalid parent comment ID", 404));
            }
        }

        // TODO: scrape comment text for @mentions and send notifs
        let new_comment = CommentForm {
            creator_id: user.id,
            body: Some(data.body),
            post_id: data.post_id,
            parent_id: data.parent_id,
            ..CommentForm::default()
        };

        let new_comment = blocking(context.pool(), move |conn| {
            Comment::submit(conn, new_comment)
        })
        .await??;

        // auto upvote own comment
        let comment_like = CommentLikeForm {
            user_id: user.id,
            comment_id: new_comment.id,
            score: 1,
        };

        blocking(context.pool(), move |conn| {
            CommentLike::vote(conn, &comment_like)
        })
        .await??;

        Ok(new_comment)
    }
}
