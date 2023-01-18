use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    sensitive::Sensitive,
    user::{Login, LoginResponse},
    utils::blocking,
};
use tinyboards_db::models::{user::users::User, site::site::Site};
use tinyboards_db_views::structs::UserView;
use tinyboards_utils::{error::TinyBoardsError, passhash::verify_password};

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for Login {
    type Response = LoginResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        _: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let (user, user_view) = blocking(context.pool(), move |conn| {
            let user: User = if self.username_or_email.contains('@') {
                User::get_by_email(conn, &self.username_or_email)
            } else {
                User::get_by_name(conn, &self.username_or_email)
            }?;

            let user_view: UserView = UserView::read(conn, user.id)?;

            Ok::<(User, UserView), TinyBoardsError>((user, user_view))
        })
        .await??;

        let site = blocking(context.pool(), move |conn| {
            Site::read_local(conn)
        }).await??;

        if site.require_application == true && user.is_application_accepted == false {
            return Err(TinyBoardsError::from_message(401, "login failed - application not accepted"));
        }

        if !verify_password(&user.passhash, &self.password) {
            return Err(TinyBoardsError::from_message(400, "login failed"));
        }

        Ok(LoginResponse {
            jwt: Sensitive::new(user.get_jwt(&context.master_key().jwt)),
            user: user_view,
        })
    }
}
