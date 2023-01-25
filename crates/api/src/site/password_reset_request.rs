use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{PasswordResetRequest},
    utils::{
        blocking,
        send_password_reset_email,
    },
};
use tinyboards_db::models::{site::password_resets::*, user::users::User};
use tinyboards_db::traits::Crud;
use tinyboards_utils::{error::TinyBoardsError, utils::generate_rand_string};

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for PasswordResetRequest {
    type Response = ();
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        _: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        let data: &PasswordResetRequest = &self;
        let email = data.email.clone();

        let user_res = blocking(context.pool(), move |conn| {
            User::get_by_email(conn, &email)
        })
        .await?;

        if user_res.is_err() {
            return Err(TinyBoardsError::from_message(404, "user not found"));
        } else {

            let user = user_res.unwrap();
            let user_id = user.id;
            let reset_token = generate_rand_string();
            
            let form = PasswordResetForm {
                user_id,
                reset_token,
            };
            // create the password reset in the database
            let reset_request = blocking(context.pool(), move |conn| {
                PasswordReset::create(conn, &form)
            })
            .await??;
            // send the email to the user with the password reset link
            let reset_link = format!("{}/password_reset/{}", context.settings().get_protocol_and_hostname(), reset_request.reset_token);
            // send password reset email
            send_password_reset_email(&user.name, &data.email, &reset_link, context.settings()).await?;
        }
        Ok(())
    }
}