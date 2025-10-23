use async_graphql::*;
use tinyboards_db::{
    models::{
        comment::comment_report::CommentReport,
        post::post_report::PostReport,
        board::board_mods::{BoardModerator, ModPerms},
        user::user::AdminPerms,
        post::posts::Post,
        comment::comments::Comment,
    },
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

use crate::LoggedInUser;

#[derive(Default)]
pub struct ModerationQueueQueries;

#[derive(SimpleObject)]
pub struct ModerationQueueItem {
    pub id: i32,
    pub item_type: String, // "post_report", "comment_report", "pending_post", "pending_comment"
    pub content_id: i32,
    pub reporter_id: Option<i32>,
    pub reason: Option<String>,
    pub content_preview: String,
    pub creation_date: String,
    pub board_id: i32,
    pub priority: i32, // Higher number = higher priority
}

#[derive(SimpleObject)]
pub struct ModerationQueue {
    pub items: Vec<ModerationQueueItem>,
    pub total_count: i32,
    pub pending_reports: i32,
    pub pending_content: i32,
}

#[Object]
impl ModerationQueueQueries {
    /// Get moderation queue (admin/moderator only)
    pub async fn get_moderation_queue(
        &self,
        ctx: &Context<'_>,
        board_id: Option<i32>,
        item_type: Option<String>, // Filter by type: "reports", "pending_content", "all"
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<ModerationQueue> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);
        let item_type = item_type.unwrap_or_else(|| "all".to_string());

        // Check if user is admin or moderator
        let is_admin = user.has_permission(AdminPerms::Content);

        if !is_admin && board_id.is_none() {
            return Err(TinyBoardsError::from_message(
                403,
                "You must specify a board_id if you're not an admin",
            )
            .into());
        }

        // If board_id is specified, check if user is a moderator of that board
        if let Some(board_id) = board_id {
            if !is_admin {
                match BoardModerator::get_by_user_id_for_board(pool, user.id, board_id, true).await {
                    Ok(moderator) => {
                        if !moderator.has_permission(ModPerms::Content) {
                            return Err(TinyBoardsError::from_message(
                                403,
                                "You don't have permission to view the moderation queue for this board",
                            )
                            .into());
                        }
                    }
                    Err(_) => {
                        return Err(TinyBoardsError::from_message(
                            403,
                            "You are not a moderator of this board",
                        )
                        .into());
                    }
                }
            }
        }

        let mut items = Vec::new();

        // Get post reports if requested
        if item_type == "all" || item_type == "reports" {
            let post_reports = PostReport::list(
                pool,
                board_id,
                Some(false), // Only unresolved
                Some(limit),
                Some(offset),
            ).await?;

            for report in post_reports {
                items.push(ModerationQueueItem {
                    id: report.id,
                    item_type: "post_report".to_string(),
                    content_id: report.post_id,
                    reporter_id: Some(report.creator_id),
                    reason: Some(report.reason),
                    content_preview: report.original_post_title,
                    creation_date: report.creation_date.to_string(),
                    board_id: 0, // Will be filled from post data if needed
                    priority: 2, // Reports have high priority
                });
            }
        }

        // Get comment reports if requested
        if item_type == "all" || item_type == "reports" {
            let comment_reports = CommentReport::list(
                pool,
                board_id,
                Some(false), // Only unresolved
                Some(limit),
                Some(offset),
            ).await?;

            for report in comment_reports {
                let preview = if report.original_comment_text.len() > 100 {
                    format!("{}...", &report.original_comment_text[0..100])
                } else {
                    report.original_comment_text
                };

                items.push(ModerationQueueItem {
                    id: report.id,
                    item_type: "comment_report".to_string(),
                    content_id: report.comment_id,
                    reporter_id: Some(report.creator_id),
                    reason: Some(report.reason),
                    content_preview: preview,
                    creation_date: report.creation_date.to_string(),
                    board_id: 0, // Will be filled from comment data if needed
                    priority: 2, // Reports have high priority
                });
            }
        }

        // Get pending posts if requested
        if item_type == "all" || item_type == "pending_content" {
            let pending_posts = Post::get_pending_approval(
                pool,
                board_id,
                Some(limit),
                Some(offset),
            ).await?;

            for post in pending_posts {
                let preview = if post.title.len() > 100 {
                    format!("{}...", &post.title[0..100])
                } else {
                    post.title.clone()
                };

                items.push(ModerationQueueItem {
                    id: post.id,
                    item_type: "pending_post".to_string(),
                    content_id: post.id,
                    reporter_id: None,
                    reason: None,
                    content_preview: preview,
                    creation_date: post.creation_date.to_string(),
                    board_id: post.board_id,
                    priority: 1, // Pending content has lower priority than reports
                });
            }
        }

        // Get pending comments if requested
        if item_type == "all" || item_type == "pending_content" {
            let pending_comments = Comment::get_pending_approval(
                pool,
                board_id,
                Some(limit),
                Some(offset),
            ).await?;

            for comment in pending_comments {
                let preview = if comment.body.len() > 100 {
                    format!("{}...", &comment.body[0..100])
                } else {
                    comment.body.clone()
                };

                items.push(ModerationQueueItem {
                    id: comment.id,
                    item_type: "pending_comment".to_string(),
                    content_id: comment.id,
                    reporter_id: None,
                    reason: None,
                    content_preview: preview,
                    creation_date: comment.creation_date.to_string(),
                    board_id: comment.board_id,
                    priority: 1, // Pending content has lower priority than reports
                });
            }
        }

        // Sort by priority (descending) then by creation date (descending)
        items.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| b.creation_date.cmp(&a.creation_date))
        });

        // Calculate counts
        let total_count = items.len() as i32;
        let pending_reports = items.iter()
            .filter(|item| item.item_type.ends_with("_report"))
            .count() as i32;
        let pending_content = items.iter()
            .filter(|item| item.item_type.starts_with("pending_"))
            .count() as i32;

        Ok(ModerationQueue {
            items,
            total_count,
            pending_reports,
            pending_content,
        })
    }
}