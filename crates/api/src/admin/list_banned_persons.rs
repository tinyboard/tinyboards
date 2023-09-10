use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    admin::{ListBannedPersons, ListBannedPersonsResponse},
    utils::require_user, 
};
use tinyboards_utils::error::TinyBoardsError;
use tinyboards_db_views::person_view::PersonQuery;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for ListBannedPersons {
    type Response = ListBannedPersonsResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {

        let data: &ListBannedPersons = &self;
        let limit = data.limit;
        let page = data.page;

        // require admin to view banned persons
        let _view = require_user(
            context.pool(), 
            context.master_key(), 
            auth
        )
        .await
        .require_admin()
        .unwrap()?;

        // grab the list of persons that are banned
        let person_query = PersonQuery::builder()
            .pool(context.pool())
            .is_banned(Some(true))
            .limit(limit)
            .page(page)
            .build()
            .list()
            .await?;

        Ok(ListBannedPersonsResponse {
            persons: person_query.persons,
            total_count: person_query.count,
        })
    }
}