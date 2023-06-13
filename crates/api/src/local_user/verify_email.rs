use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    user::{VerifyEmail, VerifyEmailResponse},
    utils::{send_email_verification_success},
};
use tinyboards_db::{
    models::site::email_verification::EmailVerification,
    models::person::local_user::*,
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

        let verification =  EmailVerification::read_for_token(context.pool(), &token.as_str())
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "could not find verification token"))?;

        let form = LocalUserForm {
            email_verified: Some(true),
            email: Some(Some(verification.email)),
            ..LocalUserForm::default()
        };

        let local_user_id = verification.local_user_id.clone();

        let updated_user = LocalUser::update(context.pool(), local_user_id, &form).await?;

        send_email_verification_success(&updated_user, &context.settings())?;

        Ok(VerifyEmailResponse {})
    }
}
