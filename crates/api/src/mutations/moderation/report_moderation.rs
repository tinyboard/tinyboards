use async_graphql::*;
use serde_json;
use tinyboards_db::{
    models::{
        comment::{
            comments::Comment,
            comment_report::CommentReport,
        },
        post::{
            posts::Post,
            post_report::PostReport,
        },
        board::board_mods::{BoardModerator, ModPerms},
        user::user::AdminPerms,
        moderator::moderation_log::{ModerationLog, action_types, target_types},
    },
    traits::{Crud, Reportable},
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

use crate::LoggedInUser;

#[derive(Default)]
pub struct ReportModerationMutations;

#[derive(SimpleObject)]
pub struct ResolveReportResponse {
    pub success: bool,
    pub report_id: i32,
    pub message: String,
}

#[derive(SimpleObject)]
pub struct ApproveContentResponse {
    pub success: bool,
    pub content_id: i32,
    pub message: String,
}

#[Object]
impl ReportModerationMutations {
    /// Resolve a post report (admin/moderator only)
    pub async fn resolve_post_report(
        &self,
        ctx: &Context<'_>,
        report_id: i32,
        resolution_reason: Option<String>,
    ) -> Result<ResolveReportResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        // Get the report to check permissions
        let report = PostReport::read(pool, report_id).await?;
        let post = Post::read(pool, report.post_id).await?;

        // Check if user has permission to resolve this report
        let is_admin = user.has_permission(AdminPerms::Content);
        let is_moderator = if !is_admin {
            match BoardModerator::get_by_user_id_for_board(pool, user.id, post.board_id, true).await {
                Ok(moderator) => moderator.has_permission(ModPerms::Content),
                Err(_) => false,
            }
        } else {
            true
        };

        if !is_admin && !is_moderator {
            return Err(TinyBoardsError::from_message(
                403,
                "You don't have permission to resolve this report",
            )
            .into());
        }

        // Check if report is already resolved
        if report.resolved {
            return Err(TinyBoardsError::from_message(
                400,
                "Report is already resolved",
            )
            .into());
        }

        // Resolve the report
        PostReport::resolve(pool, report_id, user.id).await?;

        // Log the moderation action
        let metadata = serde_json::json!({
            "report_reason": report.reason,
            "post_title": report.original_post_title,
            "resolution_reason": resolution_reason,
        });

        ModerationLog::log_action(
            pool,
            user.id,
            action_types::RESOLVE_REPORT,
            target_types::REPORT,
            report_id,
            Some(post.board_id),
            resolution_reason,
            Some(metadata),
            None,
        ).await?;

        Ok(ResolveReportResponse {
            success: true,
            report_id,
            message: "Post report has been resolved".to_string(),
        })
    }

    /// Resolve a comment report (admin/moderator only)
    pub async fn resolve_comment_report(
        &self,
        ctx: &Context<'_>,
        report_id: i32,
        resolution_reason: Option<String>,
    ) -> Result<ResolveReportResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        // Get the report to check permissions
        let report = CommentReport::read(pool, report_id).await?;
        let comment = Comment::read(pool, report.comment_id).await?;

        // Check if user has permission to resolve this report
        let is_admin = user.has_permission(AdminPerms::Content);
        let is_moderator = if !is_admin {
            match BoardModerator::get_by_user_id_for_board(pool, user.id, comment.board_id, true).await {
                Ok(moderator) => moderator.has_permission(ModPerms::Content),
                Err(_) => false,
            }
        } else {
            true
        };

        if !is_admin && !is_moderator {
            return Err(TinyBoardsError::from_message(
                403,
                "You don't have permission to resolve this report",
            )
            .into());
        }

        // Check if report is already resolved
        if report.resolved {
            return Err(TinyBoardsError::from_message(
                400,
                "Report is already resolved",
            )
            .into());
        }

        // Resolve the report
        CommentReport::resolve(pool, report_id, user.id).await?;

        // Log the moderation action
        let metadata = serde_json::json!({
            "report_reason": report.reason,
            "comment_text": report.original_comment_text,
            "resolution_reason": resolution_reason,
        });

        ModerationLog::log_action(
            pool,
            user.id,
            action_types::RESOLVE_REPORT,
            target_types::REPORT,
            report_id,
            Some(comment.board_id),
            resolution_reason,
            Some(metadata),
            None,
        ).await?;

        Ok(ResolveReportResponse {
            success: true,
            report_id,
            message: "Comment report has been resolved".to_string(),
        })
    }

    /// Approve a reported post (admin/moderator only)
    pub async fn approve_post(
        &self,
        ctx: &Context<'_>,
        post_id: i32,
        approval_reason: Option<String>,
    ) -> Result<ApproveContentResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        // Get the post to check permissions
        let post = Post::read(pool, post_id).await?;

        // Check if user has permission to approve this post
        let is_admin = user.has_permission(AdminPerms::Content);
        let is_moderator = if !is_admin {
            match BoardModerator::get_by_user_id_for_board(pool, user.id, post.board_id, true).await {
                Ok(moderator) => moderator.has_permission(ModPerms::Content),
                Err(_) => false,
            }
        } else {
            true
        };

        if !is_admin && !is_moderator {
            return Err(TinyBoardsError::from_message(
                403,
                "You don't have permission to approve this post",
            )
            .into());
        }

        // Update post approval status
        Post::update_approval_status(pool, post_id, "approved", Some(user.id)).await?;

        // If post was removed, unrestricted it
        if post.is_removed {
            Post::update_removed_status(pool, post_id, false).await?;
        }

        // Log the moderation action
        let metadata = serde_json::json!({
            "post_title": post.title,
            "post_creator_id": post.creator_id,
            "approval_reason": approval_reason,
        });

        ModerationLog::log_action(
            pool,
            user.id,
            action_types::APPROVE_POST,
            target_types::POST,
            post_id,
            Some(post.board_id),
            approval_reason,
            Some(metadata),
            None,
        ).await?;

        Ok(ApproveContentResponse {
            success: true,
            content_id: post_id,
            message: "Post has been approved".to_string(),
        })
    }

    /// Approve a reported comment (admin/moderator only)
    pub async fn approve_comment(
        &self,
        ctx: &Context<'_>,
        comment_id: i32,
        approval_reason: Option<String>,
    ) -> Result<ApproveContentResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        // Get the comment to check permissions
        let comment = Comment::read(pool, comment_id).await?;

        // Check if user has permission to approve this comment
        let is_admin = user.has_permission(AdminPerms::Content);
        let is_moderator = if !is_admin {
            match BoardModerator::get_by_user_id_for_board(pool, user.id, comment.board_id, true).await {
                Ok(moderator) => moderator.has_permission(ModPerms::Content),
                Err(_) => false,
            }
        } else {
            true
        };

        if !is_admin && !is_moderator {
            return Err(TinyBoardsError::from_message(
                403,
                "You don't have permission to approve this comment",
            )
            .into());
        }

        // Update comment approval status
        Comment::update_approval_status(pool, comment_id, "approved", Some(user.id)).await?;

        // If comment was removed, unremove it
        if comment.is_removed {
            Comment::update_removed_status(pool, comment_id, false).await?;
        }

        // Log the moderation action
        let metadata = serde_json::json!({
            "comment_body": comment.body,
            "comment_creator_id": comment.creator_id,
            "post_id": comment.post_id,
            "approval_reason": approval_reason,
        });

        ModerationLog::log_action(
            pool,
            user.id,
            action_types::APPROVE_COMMENT,
            target_types::COMMENT,
            comment_id,
            Some(comment.board_id),
            approval_reason,
            Some(metadata),
            None,
        ).await?;

        Ok(ApproveContentResponse {
            success: true,
            content_id: comment_id,
            message: "Comment has been approved".to_string(),
        })
    }
}