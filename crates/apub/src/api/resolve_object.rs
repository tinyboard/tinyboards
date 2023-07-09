use crate::{
    api::PerformApub,
    fetcher::search::{search_query_to_object_id, SearchableObjects},
  };
use tinyboards_federation::config::Data;
use diesel::NotFound;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{ResolveObject, ResolveObjectResponse, FederatedObject},
    utils::{check_private_instance, require_user},
};
use tinyboards_db::{utils::DbPool};
use tinyboards_db_views::structs::{CommentView, PostView, BoardView, PersonView};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait]
impl PerformApub for ResolveObject {
    type Response = ResolveObjectResponse;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(&self, context: &Data<TinyBoardsContext>, auth: Option<&str>) -> Result<ResolveObjectResponse, TinyBoardsError> {
        let data: &ResolveObject = self; 

        let view = require_user(context.pool(), context.master_key(), auth).await.unwrap()?;
        let person_id = view.person.id;

        let view = Some(view);
        check_private_instance(&view, context.pool()).await?;
        // let view = view.unwrap();

        let res = search_query_to_object_id(&data.q, context)
            .await?;

        Ok(convert_response(res, person_id, context.pool()).await?)
    }
}

async fn convert_response(
    object: SearchableObjects,
    user_id: i32,
    pool: &DbPool,
  ) -> Result<ResolveObjectResponse, TinyBoardsError> {
    use SearchableObjects::*;
    let removed_or_deleted;
    let res;
    match object {
        Person(p) => {
            removed_or_deleted = p.is_deleted;
            res = ResolveObjectResponse { object: FederatedObject::Person(Some(PersonView::read(pool, p.id).await?)) };
        },
        Board(b) => {
            removed_or_deleted = b.is_deleted || b.is_removed;
            res = ResolveObjectResponse { object: FederatedObject::Board(Some(BoardView::read(pool, b.id, Some(user_id), None).await?))};
        },
        Post(p) => {
            removed_or_deleted = p.is_deleted || p.is_removed;
            res = ResolveObjectResponse { object: FederatedObject::Post(Some(PostView::read(pool, p.id, Some(user_id), None).await?))};
        },
        Comment(c) => {
            removed_or_deleted = c.is_deleted || c.is_removed;
            res = ResolveObjectResponse { object: FederatedObject::Comment(Some(CommentView::read(pool, c.id, Some(user_id)).await?))};
        },
    };
    // if the object was deleted, don't return it.
    if removed_or_deleted {
        return Err(NotFound {}.into())
    }
    Ok(res)
  }