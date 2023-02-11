use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    user::{SearchNames, SearchNamesResponse, UsernameInfo},
    utils::{
        require_user,
    },
};
use tinyboards_db::models::user::users::User;
use tinyboards_utils::{error::TinyBoardsError};

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for SearchNames {
    type Response = SearchNamesResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>
    ) -> Result<Self::Response, TinyBoardsError> {

        require_user(context.pool(), context.master_key(), auth).await.unwrap()?;

        let user_info = User::search_by_name(context.pool(), &self.q).await?;

        let mut users: Vec<UsernameInfo> = Vec::new();

        for user in user_info
            .into_iter() {
                users.push(UsernameInfo { name: user.name, avatar: user.avatar, chat_id: user.chat_id });
        }

        Ok( SearchNamesResponse { users } )
    }
}