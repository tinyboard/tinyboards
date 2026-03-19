use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbReportStatus,
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
pub struct ReportQueries;

#[derive(SimpleObject)]
pub struct PostReportView {
    pub id: ID,
    pub creator_id: ID,
    pub post_id: ID,
    pub original_post_title: String,
    pub original_post_url: Option<String>,
    pub original_post_body: Option<String>,
    pub reason: String,
    pub status: String,
    pub resolver_id: Option<ID>,
    #[graphql(name = "createdAt")]
    pub created_at: String,
    #[graphql(name = "updatedAt")]
    pub updated_at: String,
}

#[derive(SimpleObject)]
pub struct CommentReportView {
    pub id: ID,
    pub creator_id: ID,
    pub comment_id: ID,
    pub original_comment_text: String,
    pub reason: String,
    pub status: String,
    pub resolver_id: Option<ID>,
    #[graphql(name = "createdAt")]
    pub created_at: String,
    #[graphql(name = "updatedAt")]
    pub updated_at: String,
}

fn report_status_str(status: &DbReportStatus) -> &'static str {
    match status {
        DbReportStatus::Pending => "pending",
        DbReportStatus::Resolved => "resolved",
        DbReportStatus::Dismissed => "dismissed",
    }
}

impl From<DbPostReport> for PostReportView {
    fn from(r: DbPostReport) -> Self {
        Self {
            id: r.id.to_string().into(),
            creator_id: r.creator_id.to_string().into(),
            post_id: r.post_id.to_string().into(),
            original_post_title: r.original_post_title,
            original_post_url: r.original_post_url,
            original_post_body: r.original_post_body,
            reason: r.reason,
            status: report_status_str(&r.status).to_string(),
            resolver_id: r.resolver_id.map(|id| id.to_string().into()),
            created_at: r.created_at.to_string(),
            updated_at: r.updated_at.to_string(),
        }
    }
}

impl From<DbCommentReport> for CommentReportView {
    fn from(r: DbCommentReport) -> Self {
        Self {
            id: r.id.to_string().into(),
            creator_id: r.creator_id.to_string().into(),
            comment_id: r.comment_id.to_string().into(),
            original_comment_text: r.original_comment_text,
            reason: r.reason,
            status: report_status_str(&r.status).to_string(),
            resolver_id: r.resolver_id.map(|id| id.to_string().into()),
            created_at: r.created_at.to_string(),
            updated_at: r.updated_at.to_string(),
        }
    }
}

#[Object]
impl ReportQueries {
    /// Get post reports (moderator/admin only)
    pub async fn get_post_reports(
        &self,
        ctx: &Context<'_>,
        board_id: Option<ID>,
        #[graphql(name = "statusFilter")] status_filter: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<PostReportView>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let limit = limit.unwrap_or(20).min(100);
        let offset = offset.unwrap_or(0);

        let is_admin = user.has_permission(AdminPerms::Content);

        let board_uuid: Option<Uuid> = if let Some(ref bid) = board_id {
            Some(bid.parse().map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?)
        } else {
            None
        };

        if !is_admin && board_uuid.is_none() {
            return Err(TinyBoardsError::from_message(
                403,
                "You must specify a board_id if you're not an admin",
            ).into());
        }

        // If board_id is specified, check moderator permissions
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
                    return Err(TinyBoardsError::from_message(
                        403,
                        "You don't have permission to view reports for this board",
                    ).into());
                }
            }
        }

        // Parse status filter
        let status = match status_filter.as_deref() {
            Some("resolved") => Some(DbReportStatus::Resolved),
            Some("dismissed") => Some(DbReportStatus::Dismissed),
            Some("pending") => Some(DbReportStatus::Pending),
            _ => None,
        };

        // Build query - join with posts to filter by board_id
        let mut query = post_reports::table
            .inner_join(posts::table.on(post_reports::post_id.eq(posts::id)))
            .select(post_reports::all_columns)
            .order(post_reports::created_at.desc())
            .into_boxed();

        if let Some(bid) = board_uuid {
            query = query.filter(posts::board_id.eq(bid));
        }

        if let Some(s) = status {
            query = query.filter(post_reports::status.eq(s));
        }

        let reports: Vec<DbPostReport> = query
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(reports.into_iter().map(PostReportView::from).collect())
    }

    /// Get comment reports (moderator/admin only)
    pub async fn get_comment_reports(
        &self,
        ctx: &Context<'_>,
        board_id: Option<ID>,
        #[graphql(name = "statusFilter")] status_filter: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<CommentReportView>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let limit = limit.unwrap_or(20).min(100);
        let offset = offset.unwrap_or(0);

        let is_admin = user.has_permission(AdminPerms::Content);

        let board_uuid: Option<Uuid> = if let Some(ref bid) = board_id {
            Some(bid.parse().map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?)
        } else {
            None
        };

        if !is_admin && board_uuid.is_none() {
            return Err(TinyBoardsError::from_message(
                403,
                "You must specify a board_id if you're not an admin",
            ).into());
        }

        // If board_id is specified, check moderator permissions
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
                    return Err(TinyBoardsError::from_message(
                        403,
                        "You don't have permission to view reports for this board",
                    ).into());
                }
            }
        }

        let status = match status_filter.as_deref() {
            Some("resolved") => Some(DbReportStatus::Resolved),
            Some("dismissed") => Some(DbReportStatus::Dismissed),
            Some("pending") => Some(DbReportStatus::Pending),
            _ => None,
        };

        // Build query - join with comments to filter by board_id
        let mut query = comment_reports::table
            .inner_join(comments::table.on(comment_reports::comment_id.eq(comments::id)))
            .select(comment_reports::all_columns)
            .order(comment_reports::created_at.desc())
            .into_boxed();

        if let Some(bid) = board_uuid {
            query = query.filter(comments::board_id.eq(bid));
        }

        if let Some(s) = status {
            query = query.filter(comment_reports::status.eq(s));
        }

        let reports: Vec<DbCommentReport> = query
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(reports.into_iter().map(CommentReportView::from).collect())
    }
}
