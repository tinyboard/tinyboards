use crate::{check_report_reason, Perform};
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{CreatePostReport, PostReportResponse},
    utils::{require_user, /*send_new_report_email_to_admins,*/ check_board_ban},
};
use tinyboards_db::{
    models::{
        //site::local_site::LocalSite,
        post::post_report::{PostReport, PostReportForm},
    },
    traits::Reportable,
};
use tinyboards_db_views::structs::{PostReportView, PostView};
use tinyboards_utils::error::TinyBoardsError;

/// Creates a post report and notifies the mods
#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for CreatePostReport {
    type Response = PostReportResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &CreatePostReport = &self;
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;
        //let local_site = LocalSite::read(context.pool()).await?;

        let reason = data.reason.trim();
        check_report_reason(reason)?;

        let person_id = view.person.id;
        let post_id = data.post_id;
        let post_view = PostView::read(context.pool(), post_id, None, None).await?;
        
        check_board_ban(person_id, post_view.board.id, context.pool()).await?;

        let report_form = PostReportForm {
            creator_id: Some(person_id),
            post_id: Some(post_id),
            original_post_title: Some(post_view.post.title),
            original_post_url: post_view.post.url,
            original_post_body: Some(post_view.post.body),
            reason: Some(reason.to_owned()),
            ..PostReportForm::default()
        };

        let report = PostReport::report(context.pool(), &report_form).await?;

        let post_report_view = PostReportView::read(context.pool(), report.id, Some(person_id)).await?;

        /*if local_site.reports_email_admins {
            send_new_report_email_to_admins(
                &post_report_view.creator.name,
                &post_report_view.post_creator.name,
                context.pool(),
                context.settings(),
            )
            .await?;
        }*/
        
        Ok(PostReportResponse { post_report_view })
    }
}