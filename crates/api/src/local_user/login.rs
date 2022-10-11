use crate::Perform;
use actix_web::web::Data;
use porpl_api_common::{
    data::PorplContext,
    person::{Login, LoginResponse},
    sensitive::Sensitive,
    utils::blocking,
};
use porpl_db::models::user::user::User;
use porpl_utils::error::PorplError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for Login {
    type Response = LoginResponse;
    type Route = ();

    async fn perform(
        self,
        context: &Data<PorplContext>,
        _: Self::Route,
        _: Option<&str>,
    ) -> Result<Self::Response, PorplError> {
        let u = blocking(context.pool(), move |conn| {
            if self.username_or_email.contains('@') {
                User::get_by_email(conn, &self.username_or_email)
            } else {
                User::get_by_name(conn, &self.username_or_email)
            }
        })
        .await?
        .map_err(|_| {
            PorplError::new(
                404,
                String::from(
                    "There is no account with that username/email. Consider signing up instead?",
                ),
            )
        })?;

        Ok(LoginResponse {
            jwt: Sensitive::new(u.get_jwt(context.master_key())),
        })
    }
}
