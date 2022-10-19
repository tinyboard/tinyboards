use crate::PerformCrud;
use actix_web::web::Data;
use porpl_api_common::{
    data::PorplContext,
    post::{ListPosts, ListPostsResponse},
    utils::{blocking, load_user_opt},
};
use porpl_db::{ListingType, SortType};
use porpl_db_views::post_view::PostQuery;
use porpl_utils::error::PorplError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for ListPosts {
    type Response = ListPostsResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<PorplContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<ListPostsResponse, PorplError> {
        let data: ListPosts = self;

        // check to see if user is logged in or not
        let u = load_user_opt(context.pool(), context.master_key(), auth).await?;

        let sort = data.sort.unwrap_or(SortType::Hot);
        let listing_type = data.type_.unwrap_or(ListingType::All);
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
