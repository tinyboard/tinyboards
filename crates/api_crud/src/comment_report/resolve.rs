use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    comment::{CommentReportResponse, ResolveCommentReport},
    data::TinyBoardsContext,
    utils::require_user,
};
use tinyboards_db::{
    models::{board::board_mods::ModPerms, comment::comment_report::CommentReport},
    traits::Reportable,
};
use tinyboards_db_views::structs::CommentReportView;
use tinyboards_utils::error::TinyBoardsError;

/// Resolves or unresolves a comment report and notifies the moderators
#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for ResolveCommentReport {
    type Response = CommentReportResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &ResolveCommentReport = &self;

        let report_view = CommentReportView::read(context.pool(), data.report_id, None).await?;

        let board_id = report_view.board.id;

        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(context.pool(), board_id, ModPerms::Content, None)
            .await
            .unwrap()?;

        let report_id = data.report_id;
        let person_id = view.person.id;

        if data.resolved {
            CommentReport::resolve(context.pool(), report_id, person_id).await?;
        } else {
            CommentReport::unresolve(context.pool(), report_id, person_id).await?;
        }

        let comment_report_view =
            CommentReportView::read(context.pool(), report_id, Some(person_id)).await?;

        Ok(CommentReportResponse {
            comment_report_view,
        })
    }
}
