use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    user::{VerifyEmail, VerifyEmailResponse},
    utils::{blocking, send_email_verification_success},
};
use tinyboards_db::{
    models::site::email_verification::EmailVerification,
    models::user::users::{User, UserForm},
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for VerifyEmail {
    type Response = VerifyEmailResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        _: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let token = self.token.clone();

        let verification = blocking(context.pool(), move |conn| {
            EmailVerification::read_for_token(conn, &token.as_str()).map_err(|e| {
                TinyBoardsError::from_error_message(e, "could not find verification token")
            })
        })
        .await??;

        let form = UserForm {
            email_verified: Some(true),
            email: Some(verification.email),
            ..UserForm::default()
        };

        let user_id = verification.user_id.clone();

        let updated_user = blocking(context.pool(), move |conn| {
            User::update(conn, user_id, &form)
        })
        .await??;

        send_email_verification_success(&updated_user, &context.settings())?;

        Ok(VerifyEmailResponse {})
    }
}
