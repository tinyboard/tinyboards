use crate::helpers::apub::generate_local_apub_endpoint;
use crate::helpers::apub::EndpointType;
use crate::structs::comment::Comment;
use crate::DbPool;
use crate::LoggedInUser;
use crate::Settings;
use async_graphql::*;
use tinyboards_db::models::board::board_mods::ModPerms;
use tinyboards_db::models::board::boards::Board as DbBoard;
use tinyboards_db::models::comment::comment_votes::CommentVote as DbCommentVote;
use tinyboards_db::models::comment::comment_votes::CommentVoteForm;
use tinyboards_db::models::comment::comments::Comment as DbComment;
use tinyboards_db::models::comment::comments::CommentForm;
use tinyboards_db::models::person::local_user::AdminPerms;
use tinyboards_db::models::person::person_mentions::PersonMention as DbPersonMention;
use tinyboards_db::models::person::person_mentions::PersonMentionForm;
use tinyboards_db::models::post::posts::Post as DbPost;
use tinyboards_db::traits::Crud;
use tinyboards_db::traits::Voteable;
use tinyboards_utils::parser::parse_markdown_opt;
use tinyboards_utils::utils::custom_body_parsing;
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct SubmitComment;

#[Object]
impl SubmitComment {
    pub async fn create_comment(
        &self,
        ctx: &Context<'_>,
        reply_to_post_id: Option<i32>,
        reply_to_comment_id: Option<i32>,
        body: String,
    ) -> Result<Comment> {
        let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;
        let settings = ctx.data::<Settings>()?.as_ref();

        // either a post id or a parent comment id must be provided
        if reply_to_post_id.is_none() && reply_to_comment_id.is_none() {
            return Err(TinyBoardsError::from_message(
                400,
                "You must provide either a post or comment id to reply to.",
            )
            .into());
        }

        // validate parent comment id and load parent comment, if provided
        let parent_comment = match reply_to_comment_id {
            Some(comment_id) => Some(DbComment::read(pool, comment_id).await?),
            None => None,
        };

        // validate parent post id and load parent post
        let parent_post = match parent_comment {
            Some(ref parent_comment) => DbPost::read(pool, parent_comment.post_id),
            None => DbPost::read(pool, reply_to_post_id.unwrap()), // safe unwrap because of is_none check above
        }
        .await?;

        // load parent board
        let parent_board = DbBoard::read(pool, parent_post.board_id).await?;
        // parent board must not be banned
        if parent_board.is_removed {
            return Err(TinyBoardsError::from_message(
                410,
                &format!("+{} is banned.", &parent_board.name),
            )
            .into());
        }

        // mod or admin check
        let is_mod_or_admin = if v.has_permission(AdminPerms::Content) {
            // user is admin
            true
        } else {
            // user is not admin: check mod permissions instead
            let m = DbBoard::board_get_mod(pool, parent_board.id, v.person.id).await;

            match m {
                Ok(m_opt) => match m_opt {
                    Some(m) => m.has_permission(ModPerms::Content),
                    None => false,
                },
                Err(e) => {
                    eprintln!("Error while checking mod permissions: {:?}", e);
                    false
                }
            }
        };

        // check if user is banned from board
        if !is_mod_or_admin && DbBoard::board_has_ban(pool, parent_board.id, v.person.id).await? {
            return Err(TinyBoardsError::from_message(
                410,
                &format!("You are banned from +{}.", &parent_board.name),
            )
            .into());
        }

        // further validation
        if parent_post.is_deleted {
            return Err(TinyBoardsError::from_message(410, "Parent post has been deleted.").into());
        }

        if !is_mod_or_admin && parent_post.is_removed {
            return Err(TinyBoardsError::from_message(403, "Parent post has been removed.").into());
        }

        if !is_mod_or_admin && parent_post.is_locked {
            return Err(TinyBoardsError::from_message(403, "Parent post has been locked.").into());
        }

        if let Some(ref parent_comment) = parent_comment {
            if parent_comment.is_deleted {
                return Err(
                    TinyBoardsError::from_message(410, "Parent comment has been deleted.").into(),
                );
            }

            if !is_mod_or_admin && parent_comment.is_removed {
                return Err(
                    TinyBoardsError::from_message(403, "Parent comment has been removed.").into(),
                );
            }
        }

        // top comment's level is 1
        // child comment's level is its parent's level + 1
        let level = match parent_comment {
            Some(ref parent_comment) => parent_comment.level + 1,
            None => 1,
        };

        // parse body
        let mut body_html = parse_markdown_opt(&body);
        body_html = Some(custom_body_parsing(
            &body_html.unwrap_or_default(),
            settings,
        ));

        // insert new comment into db
        let new_comment = CommentForm {
            creator_id: Some(v.person.id),
            body: Some(body),
            body_html,
            post_id: Some(parent_post.id),
            parent_id: parent_comment.as_ref().map(|c| c.id),
            board_id: Some(parent_post.board_id),
            level: Some(level),
            ..CommentForm::default()
        };

        let new_comment = DbComment::submit(pool, new_comment).await?;

        // add apub id
        let inserted_comment_id = new_comment.id;
        let protocol_and_hostname = settings.get_protocol_and_hostname();

        let apub_id = generate_local_apub_endpoint(
            EndpointType::Comment,
            &inserted_comment_id.to_string(),
            &protocol_and_hostname,
        )?;

        let update_form = CommentForm {
            ap_id: Some(apub_id),
            ..CommentForm::default()
        };

        let updated_comment = DbComment::update(pool, inserted_comment_id, &update_form).await?;

        // auto upvote own comment
        let comment_vote = CommentVoteForm {
            person_id: v.person.id,
            comment_id: updated_comment.id,
            score: 1,
            post_id: updated_comment.post_id,
        };

        DbCommentVote::vote(pool, &comment_vote).await?;

        //let new_comment =
        //   CommentView::read(context.pool(), new_comment.id, Some(view.person.id)).await?;

        // send notifications
        //let mentions = scrape_text_for_mentions(&new_comment.comment.body_html);
        //let recipient_ids =
        //   send_notifications(mentions, &new_comment.comment, &view.person, &post, context)
        //       .await?;

        // if parent comment has person_mentions then mark them as read
        if let Some(ref parent_comment) = parent_comment {
            let person_id = v.person.id;
            let person_mention =
                DbPersonMention::read_by_comment_and_person(pool, parent_comment.id, person_id)
                    .await;
            if let Ok(mention) = person_mention {
                DbPersonMention::update(
                    pool,
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

        let comment = DbComment::get_with_counts(pool, updated_comment.id).await?;
        Ok(Comment::from(comment))
    }
}
