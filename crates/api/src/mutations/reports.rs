use async_graphql::*;
use tinyboards_db::{
    models::{
        comment::{
            comments::Comment,
            comment_report::{CommentReport, CommentReportForm},
        },
        post::{
            posts::Post,
            post_report::{PostReport, PostReportForm},
        },
    },
    traits::{Crud, Reportable},
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

use crate::LoggedInUser;

#[derive(Default)]
pub struct ReportMutations;

#[derive(SimpleObject)]
pub struct ReportResponse {
    pub success: bool,
    pub report_id: i32,
}

#[Object]
impl ReportMutations {
    /// Report a post
    pub async fn report_post(
        &self,
        ctx: &Context<'_>,
        post_id: i32,
        reason: String,
    ) -> Result<ReportResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        if reason.trim().len() < 3 {
            return Err(TinyBoardsError::from_message(
                400,
                "Report reason must be at least 3 characters",
            )
            .into());
        }

        if reason.len() > 500 {
            return Err(TinyBoardsError::from_message(
                400,
                "Report reason cannot exceed 500 characters",
            )
            .into());
        }

        // Verify post exists
        let post = Post::read(pool, post_id).await?;
        
        // Don't allow reporting own posts
        if post.creator_id == user.id {
            return Err(TinyBoardsError::from_message(
                400,
                "You cannot report your own post",
            )
            .into());
        }

        let form = PostReportForm {
            creator_id: Some(user.id),
            post_id: Some(post_id),
            original_post_title: Some(post.title),
            original_post_url: post.url,
            original_post_body: Some(post.body),
            reason: Some(reason),
            ..Default::default()
        };

        let report = PostReport::report(pool, &form).await?;
        
        Ok(ReportResponse {
            success: true,
            report_id: report.id,
        })
    }

    /// Report a comment
    pub async fn report_comment(
        &self,
        ctx: &Context<'_>,
        comment_id: i32,
        reason: String,
    ) -> Result<ReportResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        if reason.trim().len() < 3 {
            return Err(TinyBoardsError::from_message(
                400,
                "Report reason must be at least 3 characters",
            )
            .into());
        }

        if reason.len() > 500 {
            return Err(TinyBoardsError::from_message(
                400,
                "Report reason cannot exceed 500 characters",
            )
            .into());
        }

        // Verify comment exists
        let comment = Comment::read(pool, comment_id).await?;
        
        // Don't allow reporting own comments
        if comment.creator_id == user.id {
            return Err(TinyBoardsError::from_message(
                400,
                "You cannot report your own comment",
            )
            .into());
        }

        let form = CommentReportForm {
            creator_id: Some(user.id),
            comment_id: Some(comment_id),
            original_comment_text: Some(comment.body),
            reason: Some(reason),
            ..Default::default()
        };

        let report = CommentReport::report(pool, &form).await?;
        
        Ok(ReportResponse {
            success: true,
            report_id: report.id,
        })
    }
}