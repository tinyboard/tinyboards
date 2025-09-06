use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    person::{GetPersonDetails, GetPersonDetailsResponse},
    utils::{check_private_instance, load_user_opt},
};
use tinyboards_db_views::structs::{PersonView};
use tinyboards_utils::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetPersonDetails {
    type Response = GetPersonDetailsResponse;
    type Route = ();

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _path: Self::Route,
        auth: Option<&str>,
    ) -> Result<GetPersonDetailsResponse, TinyBoardsError> {
        let data = self;
        let v = load_user_opt(context.pool(), context.master_key(), auth).await?;
        
        // check to see if instance is set to private before listing person
        check_private_instance(&v, context.pool()).await?;

        // Use person_id from the request data
        let person_id = data.person_id.ok_or_else(|| {
            TinyBoardsError::from_message(400, "person_id is required")
        })?;
        
        let person_view = PersonView::read(context.pool(), person_id, false).await?;

        // For local-only mode, return basic person details without posts/comments
        Ok(GetPersonDetailsResponse {
            person_view,
            posts: vec![],
            comments: vec![],
            comments_count_total: 0,
            posts_count_total: 0,
            moderates: vec![],
        })
    }
}