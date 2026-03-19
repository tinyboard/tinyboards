use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::{DbApprovalStatus, DbReportStatus},
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        comment::comment_report::CommentReport as DbCommentReport,
        post::post_report::PostReport as DbPostReport,
        user::user::AdminPerms,
    },
    schema::{board_moderators, comment_reports, comments, post_reports, posts},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::LoggedInUser;

#[derive(Default)]
pub struct ModerationQueueQueries;

#[derive(SimpleObject)]
pub struct ModerationQueueItem {
    pub id: ID,
    pub item_type: String,
    pub content_id: ID,
    pub reporter_id: Option<ID>,
    pub reason: Option<String>,
    pub content_preview: String,
    #[graphql(name = "createdAt")]
    pub created_at: String,
    pub board_id: Option<ID>,
    pub priority: i32,
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
    /// Get moderation queue combining pending reports and unresolved content
    pub async fn get_moderation_queue(
        &self,
        ctx: &Context<'_>,
        board_id: Option<ID>,
        item_type: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<ModerationQueue> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);
        let item_type = item_type.unwrap_or_else(|| "all".to_string());

        let board_uuid: Option<Uuid> = if let Some(ref bid) = board_id {
            Some(bid.parse().map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?)
        } else {
            None
        };

        // Permission check
        let is_admin = user.has_permission(AdminPerms::Content);
        if !is_admin && board_uuid.is_none() {
            return Err(TinyBoardsError::from_message(403, "You must specify a board_id if you're not an admin").into());
        }

        if let Some(bid) = board_uuid {
            if !is_admin {
                let moderator: BoardModerator = board_moderators::table
                    .filter(board_moderators::board_id.eq(bid))
                    .filter(board_moderators::user_id.eq(user.id))
                    .filter(board_moderators::is_invite_accepted.eq(true))
                    .first(conn)
                    .await
                    .map_err(|_| TinyBoardsError::from_message(403, "You are not a moderator of this board"))?;

                if !moderator.has_permission(ModPerms::Content) {
                    return Err(TinyBoardsError::from_message(403, "Insufficient moderation permissions").into());
                }
            }
        }

        let mut items = Vec::new();

        // Get pending post reports
        if item_type == "all" || item_type == "reports" {
            let mut query = post_reports::table
                .inner_join(posts::table.on(post_reports::post_id.eq(posts::id)))
                .filter(post_reports::status.eq(DbReportStatus::Pending))
                .select((post_reports::all_columns, posts::board_id))
                .order(post_reports::created_at.desc())
                .into_boxed();

            if let Some(bid) = board_uuid {
                query = query.filter(posts::board_id.eq(bid));
            }

            let post_reports_with_board: Vec<(DbPostReport, Uuid)> = query
                .limit(limit)
                .offset(offset)
                .load(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            for (report, bid) in post_reports_with_board {
                items.push(ModerationQueueItem {
                    id: report.id.to_string().into(),
                    item_type: "post_report".to_string(),
                    content_id: report.post_id.to_string().into(),
                    reporter_id: Some(report.creator_id.to_string().into()),
                    reason: Some(report.reason),
                    content_preview: report.original_post_title,
                    created_at: report.created_at.to_string(),
                    board_id: Some(bid.to_string().into()),
                    priority: 2,
                });
            }

            // Get pending comment reports
            let mut query = comment_reports::table
                .inner_join(comments::table.on(comment_reports::comment_id.eq(comments::id)))
                .filter(comment_reports::status.eq(DbReportStatus::Pending))
                .select((comment_reports::all_columns, comments::board_id))
                .order(comment_reports::created_at.desc())
                .into_boxed();

            if let Some(bid) = board_uuid {
                query = query.filter(comments::board_id.eq(bid));
            }

            let comment_reports_with_board: Vec<(DbCommentReport, Uuid)> = query
                .limit(limit)
                .offset(offset)
                .load(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            for (report, bid) in comment_reports_with_board {
                let preview = if report.original_comment_text.len() > 100 {
                    format!("{}...", &report.original_comment_text[..100])
                } else {
                    report.original_comment_text
                };

                items.push(ModerationQueueItem {
                    id: report.id.to_string().into(),
                    item_type: "comment_report".to_string(),
                    content_id: report.comment_id.to_string().into(),
                    reporter_id: Some(report.creator_id.to_string().into()),
                    reason: Some(report.reason),
                    content_preview: preview,
                    created_at: report.created_at.to_string(),
                    board_id: Some(bid.to_string().into()),
                    priority: 2,
                });
            }
        }

        // Get pending posts awaiting approval
        if item_type == "all" || item_type == "pending_content" {
            let mut query = posts::table
                .filter(posts::approval_status.eq(DbApprovalStatus::Pending))
                .order(posts::created_at.desc())
                .into_boxed();

            if let Some(bid) = board_uuid {
                query = query.filter(posts::board_id.eq(bid));
            }

            let pending_posts: Vec<tinyboards_db::models::post::posts::Post> = query
                .limit(limit)
                .offset(offset)
                .load(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            for post in pending_posts {
                let preview = if post.title.len() > 100 {
                    format!("{}...", &post.title[..100])
                } else {
                    post.title.clone()
                };

                items.push(ModerationQueueItem {
                    id: post.id.to_string().into(),
                    item_type: "pending_post".to_string(),
                    content_id: post.id.to_string().into(),
                    reporter_id: None,
                    reason: None,
                    content_preview: preview,
                    created_at: post.created_at.to_string(),
                    board_id: Some(post.board_id.to_string().into()),
                    priority: 1,
                });
            }
        }

        // Get pending comments awaiting approval
        if item_type == "all" || item_type == "pending_content" {
            let mut query = comments::table
                .filter(comments::approval_status.eq(DbApprovalStatus::Pending))
                .order(comments::created_at.desc())
                .into_boxed();

            if let Some(bid) = board_uuid {
                query = query.filter(comments::board_id.eq(bid));
            }

            let pending_comments: Vec<tinyboards_db::models::comment::comments::Comment> = query
                .limit(limit)
                .offset(offset)
                .load(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            for comment in pending_comments {
                let preview = if comment.body.len() > 100 {
                    format!("{}...", &comment.body[..100])
                } else {
                    comment.body.clone()
                };

                items.push(ModerationQueueItem {
                    id: comment.id.to_string().into(),
                    item_type: "pending_comment".to_string(),
                    content_id: comment.id.to_string().into(),
                    reporter_id: None,
                    reason: None,
                    content_preview: preview,
                    created_at: comment.created_at.to_string(),
                    board_id: Some(comment.board_id.to_string().into()),
                    priority: 1,
                });
            }
        }

        // Sort by priority (descending) then by created_at (descending)
        items.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| b.created_at.cmp(&a.created_at))
        });

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
