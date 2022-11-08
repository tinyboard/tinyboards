use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    sensitive::Sensitive,
    user::{Login, LoginResponse},
    utils::blocking,
};
use tinyboards_db::models::user::user::User;
use tinyboards_db_views::structs::UserView;
use tinyboards_utils::{error::TinyBoardsError, passhash::verify_password};

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
        let (user, user_view) = blocking(context.pool(), move |conn| {
            let user = if self.username_or_email.contains('@') {
                User::get_by_email(conn, &self.username_or_email)
            } else {
                User::get_by_name(conn, &self.username_or_email)
            }
            .map_err(|_| TinyBoardsError::new(403, String::from("Login failed")))?;

            let user_view =
                UserView::read(conn, user.id).map_err(|_| TinyBoardsError::err_500())?;

            Ok((user, user_view))
        })
        .await??;

        if !verify_password(&user.passhash, &self.password) {
            return Err(TinyBoardsError::new(403, String::from("Login failed")));
        }

        Ok(LoginResponse {
            jwt: Sensitive::new(user.get_jwt(&context.master_key().jwt)),
            user: user_view,
        })
    }
}
