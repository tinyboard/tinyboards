use crate::PerformCrud;
use actix_web::web::Data;
use porpl_api_common::{
    data::PorplContext,
    user::{GetUser, GetUserNamePath},
    utils::blocking,
};
use porpl_db::models::user::user::User;
use porpl_utils::PorplError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for GetUser {
    type Response = User;
    type Route = GetUserNamePath;

    async fn perform(
        self,
        context: &Data<PorplContext>,
        path: Self::Route,
        _: Option<&str>,
    ) -> Result<User, PorplError> {
        blocking(context.pool(), move |conn| {
            User::get_by_name(conn, &path.username)
        })
        .await?
        .map_err(|_| PorplError::new(404, String::from("No user with that name found")))
    }
}
