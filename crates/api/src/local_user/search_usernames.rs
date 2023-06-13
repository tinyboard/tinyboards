use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    user::{SearchNames, SearchNamesResponse, UsernameInfo},
    utils::{
        require_user,
    },
};
use tinyboards_db::models::person::person::Person;
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

        let person_info = Person::search_by_name(context.pool(), &self.q).await?;

        let mut users: Vec<UsernameInfo> = Vec::new();

        for person in person_info
            .into_iter() {
                users.push(UsernameInfo { name: person.name, avatar: person.avatar });
        }

        Ok( SearchNamesResponse { users } )
    }
}