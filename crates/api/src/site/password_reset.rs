use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{ExecutePasswordReset, PasswordResetTokenPath},
    utils::{
        send_password_reset_success_email,
    },
};
use tinyboards_db::models::{site::password_resets::PasswordReset, local_user::users::User};
use tinyboards_db::traits::Crud;
use tinyboards_utils::{error::TinyBoardsError, hash_password, passhash::verify_password};

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for ExecutePasswordReset {
    type Response = ();
    type Route = PasswordResetTokenPath;

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        _: Option<&str>,
    ) -> Result<(), TinyBoardsError> {

        let data: &ExecutePasswordReset = &self;
        let reset_token = path.reset_token.clone();
        let new_password = data.new_password.clone();
        let new_password_verify = data.new_password_verify.clone();

        let reset_request = PasswordReset::get_by_token(context.pool(), &reset_token).await?;
        
        if new_password != new_password_verify {
            return Err(TinyBoardsError::from_message(400, "passwords did not match"));
        }

        let user = User::read(context.pool(), reset_request.person_id.clone()).await?;

        let equals_old_password = verify_password(&user.passhash, &new_password);

        if equals_old_password == true {
            return Err(TinyBoardsError::from_message(400, "use a new password"));
        }

        let new_passhash = hash_password(new_password);
        
        // actually update the password here
        User::update_passhash(context.pool(), user.id.clone(), new_passhash).await?;

        // no longer need the password reset in the db, so delete it here
        PasswordReset::delete(context.pool(), reset_request.id).await?;

        // send an email that the reset was successful
        send_password_reset_success_email(&user.name, &user.email.unwrap(), context.settings()).await?;

        Ok(())
    }
}
