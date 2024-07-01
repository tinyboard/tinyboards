use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    comment::{GetCommentReports, ListCommentReports, ListCommentReportsResponse},
    data::TinyBoardsContext,
    utils::require_user,
};
use tinyboards_db::models::{board::board_mods::ModPerms, person::local_user::AdminPerms};
use tinyboards_db_views::comment_report_view::CommentReportQuery;
use tinyboards_utils::error::TinyBoardsError;

/// Lists comment reports for a board if an id is supplied
/// or returns all comment reports for a board that a user moderates
#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for ListCommentReports {
    type Response = ListCommentReportsResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &ListCommentReports = &self;

        // require board mod at least to view reports
        let mut user_res = require_user(context.pool(), context.master_key(), auth).await;

        if let Some(board_id) = data.board_id {
            user_res = user_res
                .require_board_mod(context.pool(), board_id, ModPerms::Content)
                .await;
        } else {
            user_res = user_res.require_admin(AdminPerms::Content);
        }

        let view_res = user_res.unwrap();

        if view_res.is_ok() {
            let view = view_res?;
            let person_id = view.person.id;
            let admin = view.person.is_admin;
            let board_id = data.board_id;
            let unresolved_only = data.unresolved_only;
            let page = data.page;
            let limit = data.limit;

            let report_reponse = CommentReportQuery::builder()
                .pool(context.pool())
                .my_person_id(person_id)
                .admin(admin)
                .board_id(board_id)
                .unresolved_only(unresolved_only)
                .page(page)
                .limit(limit)
                .build()
                .list()
                .await?;

            Ok(ListCommentReportsResponse {
                reports: report_reponse.reports,
                total_count: report_reponse.count,
            })
        } else {
            return Err(TinyBoardsError::from_message(
                403,
                "need to be at least a board moderator to list reports.",
            ));
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for GetCommentReports {
    type Response = ListCommentReportsResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &Self = &self;

        let v = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin(AdminPerms::Content)
            .unwrap()?;

        let query_response = CommentReportQuery::builder()
            .pool(context.pool())
            .my_person_id(v.person.id)
            .admin(true)
            .unresolved_only(data.unresolved_only)
            .comment_id(Some(data.comment_id))
            .page(Some(1))
            .build()
            .list()
            .await?;

        Ok(ListCommentReportsResponse {
            reports: query_response.reports,
            total_count: query_response.count,
        })
    }
}
