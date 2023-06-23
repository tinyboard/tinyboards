use crate::{check_report_reason, Perform};
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    comment::{CreateCommentReport, CommentReportResponse},
    utils::{require_user, send_new_report_email_to_admins, check_board_ban},
};
use tinyboards_db::{
    models::{
        site::local_site::LocalSite,
        comment::comment_report::{CommentReport, CommentReportForm},
    },
    traits::Reportable,
};
use tinyboards_db_views::structs::{CommentReportView, CommentView};
use tinyboards_utils::error::TinyBoardsError;

/// Creates a comment report and notifies the mods
#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for CreateCommentReport {
    type Response = CommentReportResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &CreateCommentReport = &self;
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;
        let local_site = LocalSite::read(context.pool()).await?;

        let reason = data.reason.trim();
        check_report_reason(reason)?;

        let person_id = view.person.id;
        let comment_id = data.comment_id;
        let comment_view = CommentView::read(context.pool(), comment_id, None).await?;
        
        check_board_ban(person_id, comment_view.board.id, context.pool()).await?;

        let report_form = CommentReportForm {
            creator_id: Some(person_id),
            comment_id: Some(comment_id),
            original_comment_text: Some(comment_view.comment.body),
            reason: Some(reason.to_owned()),
            ..CommentReportForm::default()
        };

        let report = CommentReport::report(context.pool(), &report_form).await?;

        let comment_report_view = CommentReportView::read(context.pool(), report.id, Some(person_id)).await?;

        if local_site.reports_email_admins {
            send_new_report_email_to_admins(
                &comment_report_view.creator.name,
                &comment_report_view.comment_creator.name,
                context.pool(),
                context.settings(),
            )
            .await?;
        }
        
        Ok(CommentReportResponse { comment_report_view })    
    }
}