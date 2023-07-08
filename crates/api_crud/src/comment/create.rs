use crate::PerformCrud;
use actix_web::web;
use tinyboards_api_common::{
    comment::CreateComment,
    data::TinyBoardsContext,
    utils::{
        check_board_deleted_or_removed, check_post_deleted_removed_or_locked,
        generate_local_apub_endpoint, require_user, EndpointType,
    },
    websocket::send::send_notifications,
};
use tinyboards_db::{
    models::{
        apub::actor_language::BoardLanguage,
        comment::{
            comment_votes::{CommentVote, CommentVoteForm},
            comments::{Comment, CommentForm},
        },
        person::person_mentions::{PersonMention, PersonMentionForm},
        post::posts::Post,
    },
    traits::{Crud, Voteable},
};
use tinyboards_db_views::structs::CommentView;
use tinyboards_utils::{
    parser::parse_markdown,
    utils::{custom_body_parsing, scrape_text_for_mentions},
    TinyBoardsError,
};

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for CreateComment {
    type Response = CommentView;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &web::Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<CommentView, TinyBoardsError> {
        let data = self;

        let post = Post::read(context.pool(), data.post_id).await?;

        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .not_banned()
            .not_banned_from_board(post.board_id, context.pool())
            .await
            .unwrap()?;

        // checks to see if the board even exists in the first place
        check_board_deleted_or_removed(post.board_id, context.pool()).await?;

        // checks to see if the post was deleted, removed, or locked
        check_post_deleted_removed_or_locked(post.id, context.pool()).await?;

        // check if parent comment exists
        // TODO: check if post's op is blocking the user (?)
        if Post::check_if_exists(context.pool(), data.post_id)
            .await?
            .is_none()
        {
            return Err(TinyBoardsError::from_message(400, "invalid post id"));
        }

        let mut level = 1;
        // check if parent comment exists, if provided
        // TODO: check if comment's author is blocking the user (?)
        if let Some(cid) = data.parent_id {
            let parent_comment = Comment::get_by_id(context.pool(), cid).await?;
            if parent_comment.is_none() {
                return Err(TinyBoardsError::from_message(
                    400,
                    "invalid parent comment id",
                ));
            }

            // we can unwrap safely, because the above check made sure to abort if the comment is None
            // abort if the comment the user is replying to doesn't belong to the specified post - may be useful later
            let parent_comment = parent_comment.unwrap();
            if parent_comment.post_id != data.post_id {
                return Err(TinyBoardsError::from_message(400, "bad request"));
            }

            if (parent_comment.is_removed || parent_comment.is_deleted || parent_comment.is_locked)
                && !view.local_user.is_admin
            {
                return Err(TinyBoardsError::from_message(
                    403,
                    "Comment deleted, removed or locked, you can't reply to it anymore.",
                ));
            }

            level = parent_comment.level + 1;
        }

        let mut body_html = parse_markdown(&data.body);
        body_html = Some(custom_body_parsing(
            &body_html.unwrap_or_default(),
            context.settings(),
        ));

        let parent_opt = if let Some(parent_id) = data.parent_id {
            Comment::read(context.pool(), parent_id).await.ok()
        } else {
            None
        };

        if let Some(parent) = parent_opt.as_ref() {
            if parent.post_id != data.post_id {
                return Err(TinyBoardsError::from_message(
                    400,
                    "could not create comment",
                ));
            }
            // check comment depth here?
        }

        // if no language is set then copy from parent post/comment
        let parent_language = parent_opt
            .as_ref()
            .map(|p| p.language_id)
            .unwrap_or(post.language_id);

        let language_id = data.language_id.unwrap_or(parent_language);

        BoardLanguage::is_allowed_board_language(context.pool(), Some(language_id), post.board_id)
            .await?;

        let new_comment = CommentForm {
            creator_id: Some(view.person.id),
            body: Some(data.body),
            body_html,
            post_id: Some(data.post_id),
            parent_id: data.parent_id,
            board_id: Some(post.board_id),
            level: Some(level),
            language_id: Some(language_id),
            ..CommentForm::default()
        };

        let new_comment = Comment::submit(context.pool(), new_comment).await?;

        // add apub id
        let inserted_comment_id = new_comment.id;
        let protocol_and_hostname = context.settings().get_protocol_and_hostname();

        let apub_id = generate_local_apub_endpoint(
            EndpointType::Comment,
            &inserted_comment_id.to_string(),
            &protocol_and_hostname,
        )?;

        let update_form = CommentForm {
            ap_id: Some(apub_id),
            ..CommentForm::default()
        };

        let updated_comment =
            Comment::update(context.pool(), inserted_comment_id, &update_form).await?;

        // auto upvote own comment
        let comment_vote = CommentVoteForm {
            person_id: view.person.id,
            comment_id: updated_comment.id,
            score: 1,
            post_id: updated_comment.post_id,
        };

        CommentVote::vote(context.pool(), &comment_vote).await?;

        let new_comment =
            CommentView::read(context.pool(), new_comment.id, Some(view.person.id)).await?;

        // send notifications
        let mentions = scrape_text_for_mentions(&new_comment.comment.body_html);
        let _recipient_ids =
            send_notifications(mentions, &new_comment.comment, &view.person, &post, context)
                .await?;

        // if parent comment has person_mentions then mark them as read
        if let Some(parent_id) = data.parent_id {
            let person_id = view.person.id;
            let person_mention =
                PersonMention::read_by_comment_and_person(context.pool(), parent_id, person_id)
                    .await;
            if let Ok(mention) = person_mention {
                PersonMention::update(
                    context.pool(),
                    mention.id,
                    &PersonMentionForm {
                        read: Some(true),
                        ..PersonMentionForm::default()
                    },
                )
                .await
                .map_err(|e| {
                    TinyBoardsError::from_error_message(e, 400, "could not update person mention")
                })?;
            }
        }

        // build comment response logic here

        Ok(new_comment)
    }
}
