use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    comment::{ListCommentReports, ListCommentReportsResponse},
    utils::require_user,
};
use tinyboards_db_views::comment_report_view::CommentReportQuery;
use tinyboards_utils::error::TinyBoardsError;

/// Lists comment reports for a board if an id is supplied
/// or returns all comment reports for a board that a user moderates
#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for ListCommentReports {
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
        let mut user_res = require_user(context.pool(), context.master_key(), auth)
            .await;

        if let Some(board_id) = data.board_id {
            user_res = user_res
                .require_board_mod(board_id, context.pool())
                .await;
        } else {
            user_res = user_res
                .require_admin();
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

            let comment_reports = CommentReportQuery::builder()
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

            Ok( ListCommentReportsResponse { comment_reports })

        } else {
            return Err(TinyBoardsError::from_message(403, "need to be at least a board moderator to list reports."));
        }
    }
}