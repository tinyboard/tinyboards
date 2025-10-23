use async_graphql::*;
use tinyboards_db::{
    models::{
        board::board_mods::BoardModerator,
        comment::comment_report::CommentReport,
        post::post_report::PostReport,
        user::user::AdminPerms,
    },
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

use crate::LoggedInUser;

#[derive(Default)]
pub struct ReportQueries;

#[derive(SimpleObject)]
pub struct PostReportView {
    pub id: i32,
    pub creator_id: i32,
    pub post_id: i32,
    pub original_post_title: String,
    pub original_post_url: Option<String>,
    pub original_post_body: Option<String>,
    pub reason: String,
    pub resolved: bool,
    pub resolver_id: Option<i32>,
    pub creation_date: String,
    pub updated: Option<String>,
}

#[derive(SimpleObject)]
pub struct CommentReportView {
    pub id: i32,
    pub creator_id: i32,
    pub comment_id: i32,
    pub original_comment_text_display: String,
    pub reason: String,
    pub resolved: bool,
    pub resolver_id: Option<i32>,
    pub creation_date: String,
    pub updated: Option<String>,
}

impl From<PostReport> for PostReportView {
    fn from(report: PostReport) -> Self {
        Self {
            id: report.id,
            creator_id: report.creator_id,
            post_id: report.post_id,
            original_post_title: report.original_post_title,
            original_post_url: report.original_post_url.map(|url| url.to_string()),
            original_post_body: report.original_post_body,
            reason: report.reason,
            resolved: report.resolved,
            resolver_id: report.resolver_id,
            creation_date: report.creation_date.to_string(),
            updated: report.updated.map(|dt| dt.to_string()),
        }
    }
}

impl From<CommentReport> for CommentReportView {
    fn from(report: CommentReport) -> Self {
        Self {
            id: report.id,
            creator_id: report.creator_id,
            comment_id: report.comment_id,
            original_comment_text_display: report.original_comment_text,
            reason: report.reason,
            resolved: report.resolved,
            resolver_id: report.resolver_id,
            creation_date: report.creation_date.to_string(),
            updated: report.updated.map(|dt| dt.to_string()),
        }
    }
}

#[Object]
impl ReportQueries {
    /// Get post reports (moderator/admin only)
    pub async fn get_post_reports(
        &self,
        ctx: &Context<'_>,
        board_id: Option<i32>,
        resolved_only: Option<bool>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<PostReportView>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        let limit = limit.unwrap_or(20).min(100);
        let offset = offset.unwrap_or(0);
        let resolved_only = resolved_only.unwrap_or(false);

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
                        if !moderator.has_permission(tinyboards_db::models::board::board_mods::ModPerms::Content) {
                            return Err(TinyBoardsError::from_message(
                                403,
                                "You don't have permission to view reports for this board",
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

        let reports = PostReport::list(
            pool,
            board_id,
            Some(resolved_only),
            Some(limit),
            Some(offset),
        )
        .await?;

        Ok(reports.into_iter().map(PostReportView::from).collect())
    }

    /// Get comment reports (moderator/admin only)
    pub async fn get_comment_reports(
        &self,
        ctx: &Context<'_>,
        board_id: Option<i32>,
        resolved_only: Option<bool>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<CommentReportView>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        let limit = limit.unwrap_or(20).min(100);
        let offset = offset.unwrap_or(0);
        let resolved_only = resolved_only.unwrap_or(false);

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
                        if !moderator.has_permission(tinyboards_db::models::board::board_mods::ModPerms::Content) {
                            return Err(TinyBoardsError::from_message(
                                403,
                                "You don't have permission to view reports for this board",
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

        let reports = CommentReport::list(
            pool,
            board_id,
            Some(resolved_only),
            Some(limit),
            Some(offset),
        )
        .await?;

        Ok(reports.into_iter().map(CommentReportView::from).collect())
    }
}