use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbReportStatus,
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        comment::comment_report::{CommentReportInsertForm, CommentReportUpdateForm},
        comment::comments::Comment as DbComment,
        post::post_report::{PostReportInsertForm, PostReportUpdateForm},
        post::posts::Post as DbPost,
        user::user::AdminPerms,
    },
    schema::{board_moderators, comment_reports, comments, post_reports, posts},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::LoggedInUser;

#[derive(Default)]
pub struct ReportMutations;

#[derive(SimpleObject)]
pub struct ReportResponse {
    pub success: bool,
    pub report_id: ID,
}

#[derive(SimpleObject)]
pub struct ResolveReportResponse {
    pub success: bool,
}

#[Object]
impl ReportMutations {
    /// Report a post
    pub async fn report_post(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        reason: String,
    ) -> Result<ReportResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        if reason.trim().len() < 3 {
            return Err(TinyBoardsError::from_message(400, "Report reason must be at least 3 characters").into());
        }
        if reason.len() > 500 {
            return Err(TinyBoardsError::from_message(400, "Report reason cannot exceed 500 characters").into());
        }

        let post_uuid: Uuid = post_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid post ID".into()))?;

        // Verify post exists
        let post: DbPost = posts::table
            .find(post_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

        if post.creator_id == user.id {
            return Err(TinyBoardsError::from_message(400, "You cannot report your own post").into());
        }

        let form = PostReportInsertForm {
            id: Uuid::new_v4(),
            creator_id: user.id,
            post_id: post_uuid,
            original_post_title: post.title,
            original_post_url: post.url,
            original_post_body: Some(post.body),
            reason,
            status: DbReportStatus::Pending,
        };

        let report: tinyboards_db::models::post::post_report::PostReport =
            diesel::insert_into(post_reports::table)
                .values(&form)
                .get_result(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(ReportResponse {
            success: true,
            report_id: report.id.to_string().into(),
        })
    }

    /// Report a comment
    pub async fn report_comment(
        &self,
        ctx: &Context<'_>,
        comment_id: ID,
        reason: String,
    ) -> Result<ReportResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        if reason.trim().len() < 3 {
            return Err(TinyBoardsError::from_message(400, "Report reason must be at least 3 characters").into());
        }
        if reason.len() > 500 {
            return Err(TinyBoardsError::from_message(400, "Report reason cannot exceed 500 characters").into());
        }

        let comment_uuid: Uuid = comment_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid comment ID".into()))?;

        // Verify comment exists
        let comment: DbComment = comments::table
            .find(comment_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Comment not found".into()))?;

        if comment.creator_id == user.id {
            return Err(TinyBoardsError::from_message(400, "You cannot report your own comment").into());
        }

        let form = CommentReportInsertForm {
            id: Uuid::new_v4(),
            creator_id: user.id,
            comment_id: comment_uuid,
            original_comment_text: comment.body,
            reason,
            status: DbReportStatus::Pending,
        };

        let report: tinyboards_db::models::comment::comment_report::CommentReport =
            diesel::insert_into(comment_reports::table)
                .values(&form)
                .get_result(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(ReportResponse {
            success: true,
            report_id: report.id.to_string().into(),
        })
    }

    /// Resolve a report (moderator/admin only)
    pub async fn resolve_report(
        &self,
        ctx: &Context<'_>,
        report_id: ID,
        report_type: String,
    ) -> Result<ResolveReportResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let report_uuid: Uuid = report_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid report ID".into()))?;

        match report_type.as_str() {
            "post" => {
                let report: tinyboards_db::models::post::post_report::PostReport =
                    post_reports::table.find(report_uuid).first(conn).await
                        .map_err(|_| TinyBoardsError::NotFound("Report not found".into()))?;

                let post: DbPost = posts::table.find(report.post_id).first(conn).await
                    .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

                check_report_permission(conn, user, post.board_id).await?;

                let form = PostReportUpdateForm {
                    status: Some(DbReportStatus::Resolved),
                    resolver_id: Some(Some(user.id)),
                    updated_at: Some(chrono::Utc::now()),
                };

                diesel::update(post_reports::table.find(report_uuid))
                    .set(&form)
                    .execute(conn)
                    .await
                    .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
            }
            "comment" => {
                let report: tinyboards_db::models::comment::comment_report::CommentReport =
                    comment_reports::table.find(report_uuid).first(conn).await
                        .map_err(|_| TinyBoardsError::NotFound("Report not found".into()))?;

                let comment: DbComment = comments::table.find(report.comment_id).first(conn).await
                    .map_err(|_| TinyBoardsError::NotFound("Comment not found".into()))?;

                check_report_permission(conn, user, comment.board_id).await?;

                let form = CommentReportUpdateForm {
                    status: Some(DbReportStatus::Resolved),
                    resolver_id: Some(Some(user.id)),
                    updated_at: Some(chrono::Utc::now()),
                };

                diesel::update(comment_reports::table.find(report_uuid))
                    .set(&form)
                    .execute(conn)
                    .await
                    .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
            }
            _ => {
                return Err(TinyBoardsError::from_message(400, "report_type must be 'post' or 'comment'").into());
            }
        }

        Ok(ResolveReportResponse { success: true })
    }

    /// Dismiss a report (moderator/admin only)
    pub async fn dismiss_report(
        &self,
        ctx: &Context<'_>,
        report_id: ID,
        report_type: String,
    ) -> Result<ResolveReportResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let report_uuid: Uuid = report_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid report ID".into()))?;

        match report_type.as_str() {
            "post" => {
                let report: tinyboards_db::models::post::post_report::PostReport =
                    post_reports::table.find(report_uuid).first(conn).await
                        .map_err(|_| TinyBoardsError::NotFound("Report not found".into()))?;

                let post: DbPost = posts::table.find(report.post_id).first(conn).await
                    .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

                check_report_permission(conn, user, post.board_id).await?;

                let form = PostReportUpdateForm {
                    status: Some(DbReportStatus::Dismissed),
                    resolver_id: Some(Some(user.id)),
                    updated_at: Some(chrono::Utc::now()),
                };

                diesel::update(post_reports::table.find(report_uuid))
                    .set(&form)
                    .execute(conn)
                    .await
                    .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
            }
            "comment" => {
                let report: tinyboards_db::models::comment::comment_report::CommentReport =
                    comment_reports::table.find(report_uuid).first(conn).await
                        .map_err(|_| TinyBoardsError::NotFound("Report not found".into()))?;

                let comment: DbComment = comments::table.find(report.comment_id).first(conn).await
                    .map_err(|_| TinyBoardsError::NotFound("Comment not found".into()))?;

                check_report_permission(conn, user, comment.board_id).await?;

                let form = CommentReportUpdateForm {
                    status: Some(DbReportStatus::Dismissed),
                    resolver_id: Some(Some(user.id)),
                    updated_at: Some(chrono::Utc::now()),
                };

                diesel::update(comment_reports::table.find(report_uuid))
                    .set(&form)
                    .execute(conn)
                    .await
                    .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
            }
            _ => {
                return Err(TinyBoardsError::from_message(400, "report_type must be 'post' or 'comment'").into());
            }
        }

        Ok(ResolveReportResponse { success: true })
    }
}

/// Check if a user has permission to resolve/dismiss reports for a board
async fn check_report_permission(
    conn: &mut diesel_async::AsyncPgConnection,
    user: &tinyboards_db::models::user::user::User,
    board_id: Uuid,
) -> Result<()> {
    let is_admin = user.has_permission(AdminPerms::Content);
    if is_admin {
        return Ok(());
    }

    let moderator: BoardModerator = board_moderators::table
        .filter(board_moderators::board_id.eq(board_id))
        .filter(board_moderators::user_id.eq(user.id))
        .filter(board_moderators::is_invite_accepted.eq(true))
        .first(conn)
        .await
        .map_err(|_| TinyBoardsError::from_message(403, "You are not a moderator of this board"))?;

    if !moderator.has_permission(ModPerms::Content) {
        return Err(TinyBoardsError::from_message(403, "You don't have permission to manage reports").into());
    }

    Ok(())
}
