use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::{DbApprovalStatus, DbModerationAction, DbReportStatus},
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        comment::comment_report::{CommentReport as DbCommentReport, CommentReportUpdateForm},
        moderator::moderation_log::ModerationLogInsertForm,
        post::post_report::{PostReport as DbPostReport, PostReportUpdateForm},
        user::user::AdminPerms,
    },
    schema::{board_moderators, comment_reports, comments, moderation_log, post_reports, posts},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::LoggedInUser;

#[derive(Default)]
pub struct ReportModerationMutations;

#[derive(SimpleObject)]
pub struct ModResolveReportResponse {
    pub success: bool,
    pub report_id: ID,
    pub message: String,
}

#[derive(SimpleObject)]
pub struct ApproveContentResponse {
    pub success: bool,
    pub content_id: ID,
    pub message: String,
}

/// Check moderator permission for a board
async fn check_mod_content_permission(
    conn: &mut diesel_async::AsyncPgConnection,
    user: &tinyboards_db::models::user::user::User,
    board_id: Uuid,
) -> Result<bool> {
    let is_admin = user.has_permission(AdminPerms::Content);
    if is_admin {
        return Ok(true);
    }

    let moderator: BoardModerator = board_moderators::table
        .filter(board_moderators::board_id.eq(board_id))
        .filter(board_moderators::user_id.eq(user.id))
        .filter(board_moderators::is_invite_accepted.eq(true))
        .first(conn)
        .await
        .map_err(|_| TinyBoardsError::from_message(403, "You are not a moderator of this board"))?;

    if !moderator.has_permission(ModPerms::Content) {
        return Err(TinyBoardsError::from_message(403, "Insufficient moderation permissions").into());
    }

    Ok(true)
}

#[Object]
impl ReportModerationMutations {
    /// Resolve a post report (admin/moderator only)
    pub async fn resolve_post_report(
        &self,
        ctx: &Context<'_>,
        report_id: ID,
        resolution_reason: Option<String>,
    ) -> Result<ModResolveReportResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let report_uuid: Uuid = report_id.parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid report ID".into()))?;

        let report: DbPostReport = post_reports::table
            .find(report_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Report not found".into()))?;

        if report.status != DbReportStatus::Pending {
            return Err(TinyBoardsError::from_message(400, "Report is already resolved").into());
        }

        // Get board_id from the post
        let board_id: Uuid = posts::table
            .find(report.post_id)
            .select(posts::board_id)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

        check_mod_content_permission(conn, user, board_id).await?;

        // Resolve the report
        diesel::update(post_reports::table.find(report_uuid))
            .set(&PostReportUpdateForm {
                status: Some(DbReportStatus::Resolved),
                resolver_id: Some(Some(user.id)),
                updated_at: Some(chrono::Utc::now()),
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Log the moderation action
        let metadata = serde_json::json!({
            "report_reason": report.reason,
            "post_title": report.original_post_title,
            "resolution_reason": resolution_reason,
        });

        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: user.id,
                action_type: DbModerationAction::RemovePost, // Closest available action
                target_type: "report".to_string(),
                target_id: report_uuid,
                board_id: Some(board_id),
                reason: resolution_reason,
                metadata: Some(metadata),
                expires_at: None,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(ModResolveReportResponse {
            success: true,
            report_id: report_uuid.to_string().into(),
            message: "Post report has been resolved".to_string(),
        })
    }

    /// Resolve a comment report (admin/moderator only)
    pub async fn resolve_comment_report(
        &self,
        ctx: &Context<'_>,
        report_id: ID,
        resolution_reason: Option<String>,
    ) -> Result<ModResolveReportResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let report_uuid: Uuid = report_id.parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid report ID".into()))?;

        let report: DbCommentReport = comment_reports::table
            .find(report_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Report not found".into()))?;

        if report.status != DbReportStatus::Pending {
            return Err(TinyBoardsError::from_message(400, "Report is already resolved").into());
        }

        let board_id: Uuid = comments::table
            .find(report.comment_id)
            .select(comments::board_id)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Comment not found".into()))?;

        check_mod_content_permission(conn, user, board_id).await?;

        diesel::update(comment_reports::table.find(report_uuid))
            .set(&CommentReportUpdateForm {
                status: Some(DbReportStatus::Resolved),
                resolver_id: Some(Some(user.id)),
                updated_at: Some(chrono::Utc::now()),
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let metadata = serde_json::json!({
            "report_reason": report.reason,
            "comment_text": report.original_comment_text,
            "resolution_reason": resolution_reason,
        });

        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: user.id,
                action_type: DbModerationAction::RemoveComment,
                target_type: "report".to_string(),
                target_id: report_uuid,
                board_id: Some(board_id),
                reason: resolution_reason,
                metadata: Some(metadata),
                expires_at: None,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(ModResolveReportResponse {
            success: true,
            report_id: report_uuid.to_string().into(),
            message: "Comment report has been resolved".to_string(),
        })
    }

    /// Approve a pending post (admin/moderator only)
    pub async fn approve_post(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        approval_reason: Option<String>,
    ) -> Result<ApproveContentResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = post_id.parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid post ID".into()))?;

        let post: tinyboards_db::models::post::posts::Post = posts::table
            .find(post_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

        check_mod_content_permission(conn, user, post.board_id).await?;

        // Update post approval status
        diesel::update(posts::table.find(post_uuid))
            .set((
                posts::approval_status.eq(DbApprovalStatus::Approved),
                posts::approved_by.eq(Some(user.id)),
            ))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // If post was removed, unrestrict it
        if post.is_removed {
            diesel::update(posts::table.find(post_uuid))
                .set(posts::is_removed.eq(false))
                .execute(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
        }

        let metadata = serde_json::json!({
            "post_title": post.title,
            "post_creator_id": post.creator_id.to_string(),
            "approval_reason": approval_reason,
        });

        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: user.id,
                action_type: DbModerationAction::RestorePost,
                target_type: "post".to_string(),
                target_id: post_uuid,
                board_id: Some(post.board_id),
                reason: approval_reason,
                metadata: Some(metadata),
                expires_at: None,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(ApproveContentResponse {
            success: true,
            content_id: post_uuid.to_string().into(),
            message: "Post has been approved".to_string(),
        })
    }

    /// Approve a pending comment (admin/moderator only)
    pub async fn approve_comment(
        &self,
        ctx: &Context<'_>,
        comment_id: ID,
        approval_reason: Option<String>,
    ) -> Result<ApproveContentResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let comment_uuid: Uuid = comment_id.parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid comment ID".into()))?;

        let comment: tinyboards_db::models::comment::comments::Comment = comments::table
            .find(comment_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Comment not found".into()))?;

        check_mod_content_permission(conn, user, comment.board_id).await?;

        // Update comment approval status
        diesel::update(comments::table.find(comment_uuid))
            .set(comments::approval_status.eq(DbApprovalStatus::Approved))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if comment.is_removed {
            diesel::update(comments::table.find(comment_uuid))
                .set(comments::is_removed.eq(false))
                .execute(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
        }

        let metadata = serde_json::json!({
            "comment_body": comment.body,
            "comment_creator_id": comment.creator_id.to_string(),
            "post_id": comment.post_id.to_string(),
            "approval_reason": approval_reason,
        });

        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: user.id,
                action_type: DbModerationAction::RestoreComment,
                target_type: "comment".to_string(),
                target_id: comment_uuid,
                board_id: Some(comment.board_id),
                reason: approval_reason,
                metadata: Some(metadata),
                expires_at: None,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(ApproveContentResponse {
            success: true,
            content_id: comment_uuid.to_string().into(),
            message: "Comment has been approved".to_string(),
        })
    }
}
