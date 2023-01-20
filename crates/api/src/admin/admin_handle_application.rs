use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    admin::{ApplicationIdPath, HandleRegistrationApplication, HandleRegistrationApplicationResponse},
    data::TinyBoardsContext,
    utils::{blocking, require_user, send_application_approval_email},
};
use tinyboards_db::{
    models::{
        site::registration_applications::{RegistrationApplication, RegistrationApplicationForm},
    },
    traits::Crud,
};
use tinyboards_db_views::structs::RegistrationApplicationView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for HandleRegistrationApplication {
    type Response = HandleRegistrationApplicationResponse;
    type Route = ApplicationIdPath;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &HandleRegistrationApplication = &self;

        // only admin should be the one to approve/deny the application
        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        let approve = data.approve.clone();
        let reason = data.reason.clone();

        let app_id = path.app_id.clone();
        
        let app = blocking(context.pool(), move |conn| {
            RegistrationApplicationView::read(conn, app_id.clone())
        })
        .await??;

        if approve == true {
            // if admin is approving the app, we can just approve the application, send an approval email, and remove it from the DB
            let app_username = app.applicant.name.clone();
            let app_email = app.applicant_settings.email.clone();
            if let Some(app_email) = app_email {
                send_application_approval_email(&app_username, &app_email, context.settings()).await?;
            }
            blocking(context.pool(), move |conn| {
                RegistrationApplication::delete(conn, app_id.clone())
            })
            .await??;

            Ok(HandleRegistrationApplicationResponse { application: None })

        } else {
            // if we are denying the application, update the app in the DB with admin who denied it and reason
            let form = RegistrationApplicationForm {
                user_id: app.application.user_id.clone(),
                answer: Some(app.application.answer.clone()),
                deny_reason: Some(reason),
                admin_id: Some(Some(user.id.clone())),
            };
            // update the application
            blocking(context.pool(), move |conn| {
                RegistrationApplication::update(conn, app_id.clone(), &form)
            })
            .await??;
            // get the updated app view
            let application = blocking(context.pool(), move |conn| RegistrationApplicationView::read(conn, app_id.clone()))
                .await??;
            
            Ok(HandleRegistrationApplicationResponse { application: Some(application) })
        }
    }
}