use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    sensitive::Sensitive,
    user::{Login, LoginResponse},
};
use tinyboards_db::models::{site::local_site::LocalSite};
use tinyboards_db_views::structs::LocalUserView;
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
                  
        let view = if self.username_or_email.contains('@') {
            LocalUserView::get_by_email(context.pool(), &self.username_or_email).await
        } else {
            LocalUserView::get_by_name(context.pool(), &self.username_or_email).await
        }?;

        let local_user_view: LocalUserView = LocalUserView::read(context.pool(), view.local_user.id).await?;

        let site = LocalSite::read(context.pool()).await?;

        if site.require_application == true && local_user_view.local_user.is_application_accepted == false {
            return Err(TinyBoardsError::from_message(401, "login failed - application not accepted"));
        }

        if !verify_password(&local_user_view.local_user.passhash, &self.password) {
            return Err(TinyBoardsError::from_message(400, "login failed"));
        }

        Ok(LoginResponse {
            jwt: Sensitive::new(local_user_view.local_user.get_jwt(&context.master_key().jwt)),
            user: local_user_view,
        })
    }
}
