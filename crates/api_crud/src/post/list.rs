use crate::PerformCrud;
use actix_web::web::Data;
use porpl_api_common::{
    data::PorplContext,
    post::{ListPosts, ListPostsResponse},
    utils::{blocking, load_user_opt},
};
use porpl_db::{
    map_to_sort_type,
    map_to_listing_type,
};
use porpl_db_views::post_view::PostQuery;
use porpl_utils::error::PorplError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for ListPosts {
    type Response = ListPostsResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<PorplContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<ListPostsResponse, PorplError> {
        let data: ListPosts = self;

        // check to see if user is logged in or not
        let u = load_user_opt(context.pool(), context.master_key(), auth).await?;

        let sort = map_to_sort_type(data.sort.as_deref());
        let listing_type = map_to_listing_type(data.listing_type.as_deref());
        let page = data.page;
        let limit = data.limit;
        let board_id = data.board_id;
        let saved_only = data.saved_only;

        let posts = blocking(context.pool(), move |conn| {
            PostQuery::builder()
                .conn(conn)
                .listing_type(Some(listing_type))
                .sort(Some(sort))
                .board_id(board_id)
                .user(u.as_ref())
                .saved_only(saved_only)
                .page(page)
                .limit(limit)
                .build()
                .list()
        })
        .await?
        .map_err(|e| {
            eprintln!("ERROR: {}", e);
            PorplError::err_500()
        })?;

        Ok(ListPostsResponse { posts })
    }
}
