use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    user::{Login, LoginResponse},
    sensitive::Sensitive,
    utils::{blocking},
};
use tinyboards_db::models::user::user::User;
use tinyboards_utils::{
    error::TinyBoardsError,
    passhash::verify_password,
};

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for Login {
    type Response = LoginResponse;
    type Route = ();

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        _: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let u = blocking(context.pool(), move |conn| {
            if self.username_or_email.contains('@') {
                User::get_by_email(conn, &self.username_or_email)
            } else {
                User::get_by_name(conn, &self.username_or_email)
            }
        })
        .await?
        .map_err(|_| TinyBoardsError::new(403, String::from("Login failed")))?;

        if !verify_password(&u.passhash, &self.password) {
            return Err(TinyBoardsError::new(403, String::from("Login failed")));
        }

        Ok(LoginResponse {
            jwt: Sensitive::new(u.get_jwt(context.master_key().jwt.as_ref())),
        })
    }
}
