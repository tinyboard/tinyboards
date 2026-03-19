use crate::helpers::files::cleanup::link_content_uploads;
use crate::helpers::permissions;
use crate::structs::comment::Comment;
use crate::{DbPool, LoggedInUser, Settings};
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbApprovalStatus,
    models::{
        aggregates::CommentAggregates,
        comment::comment_votes::CommentVoteInsertForm,
        comment::comments::{Comment as DbComment, CommentInsertForm},
        post::posts::Post as DbPost,
        site::site::Site,
    },
    schema::{board_user_bans, comment_aggregates, comment_votes, comments, posts, site},
    utils::get_conn,
};
use tinyboards_utils::{slug::generate_slug, TinyBoardsError};
use uuid::Uuid;

#[derive(Default)]
pub struct SubmitComment;

#[Object]
impl SubmitComment {
    pub async fn create_comment(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        body: String,
        parent_id: Option<ID>,
    ) -> Result<Comment> {
        let pool = ctx.data::<DbPool>()?;
        let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_approved(pool)
            .await?;
        let settings = ctx.data::<Settings>()?.as_ref();
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = post_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid post ID"))?;

        // Load the post
        let post: DbPost = posts::table
            .find(post_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

        if post.deleted_at.is_some() || post.is_removed {
            return Err(
                TinyBoardsError::from_message(404, "Post has been deleted or removed").into(),
            );
        }

        if post.is_locked {
            return Err(TinyBoardsError::from_message(403, "Post is locked").into());
        }

        // Check if user is banned from board
        let banned: bool = board_user_bans::table
            .filter(board_user_bans::board_id.eq(post.board_id))
            .filter(board_user_bans::user_id.eq(v.id))
            .first::<tinyboards_db::models::social::BoardUserBan>(conn)
            .await
            .is_ok();

        if banned {
            return Err(TinyBoardsError::from_message(403, "You are banned from this board").into());
        }

        // Resolve parent comment
        let parent_uuid: Option<Uuid> = match parent_id {
            Some(pid) => Some(
                pid.parse::<Uuid>()
                    .map_err(|_| TinyBoardsError::from_message(400, "Invalid parent comment ID"))?,
            ),
            None => None,
        };

        // Determine level based on parent
        let level = if let Some(parent_uuid) = parent_uuid {
            let parent: DbComment = comments::table
                .find(parent_uuid)
                .first(conn)
                .await
                .map_err(|_| TinyBoardsError::NotFound("Parent comment not found".into()))?;
            parent.level + 1
        } else {
            0
        };

        // Process body HTML with emojis
        let site_config: Site = site::table
            .first(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let emoji_limit = if site_config.emoji_enabled {
            site_config.max_emojis_per_post.map(|limit| limit as usize)
        } else {
            Some(0)
        };

        let body_html = crate::utils::emoji::process_content_with_emojis(
            &body,
            pool,
            Some(post.board_id),
            settings,
            emoji_limit,
        )
        .await?;

        let slug = generate_slug(&body, Some(60));
        let comment_id = Uuid::new_v4();

        let comment_form = CommentInsertForm {
            id: comment_id,
            body: body.clone(),
            body_html: body_html.clone(),
            slug,
            creator_id: v.id,
            post_id: post_uuid,
            parent_id: parent_uuid,
            board_id: post.board_id,
            language_id: None,
            level,
            approval_status: DbApprovalStatus::Approved,
            quoted_comment_id: None,
        };

        diesel::insert_into(comments::table)
            .values(&comment_form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Auto upvote own comment
        let vote_form = CommentVoteInsertForm {
            id: Uuid::new_v4(),
            user_id: v.id,
            comment_id,
            post_id: post_uuid,
            score: 1,
        };
        diesel::insert_into(comment_votes::table)
            .values(&vote_form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Link any uploaded images found in the HTML content
        if !body_html.is_empty() {
            link_content_uploads(pool, comment_id, false, &body_html).await?;
        }

        // Send notifications for mentions and replies
        use crate::helpers::notifications::{
            create_comment_reply_notification, extract_mentions, get_user_ids_for_mentions,
            create_comment_mention_notification,
        };

        // Notify parent comment author if this is a reply
        if let Some(parent_uuid) = parent_uuid {
            let parent: DbComment = comments::table
                .find(parent_uuid)
                .first(conn)
                .await
                .map_err(|_| TinyBoardsError::NotFound("Parent comment not found".into()))?;
            if parent.creator_id != v.id {
                let _ = create_comment_reply_notification(pool, parent.creator_id, comment_id)
                    .await;
            }
        }

        // Send mention notifications
        let mentions = extract_mentions(&body);
        if !mentions.is_empty() {
            if let Ok(mentioned_user_ids) = get_user_ids_for_mentions(pool, mentions).await {
                for mentioned_user_id in mentioned_user_ids {
                    if mentioned_user_id != v.id {
                        let _ =
                            create_comment_mention_notification(pool, mentioned_user_id, comment_id)
                                .await;
                    }
                }
            }
        }

        // Load the created comment with aggregates
        let db_comment: DbComment = comments::table
            .find(comment_id)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Comment not found after creation".into()))?;

        let agg: CommentAggregates = comment_aggregates::table
            .filter(comment_aggregates::comment_id.eq(comment_id))
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Comment aggregates not found".into()))?;

        Ok(Comment::from((db_comment, agg)))
    }
}
