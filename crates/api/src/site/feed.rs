use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::ListPostsResponse,
    site::GetFeed,
    utils::{check_private_instance, load_user_opt},
};
use tinyboards_db::{map_to_listing_type, map_to_sort_type, ListingType, SortType};
use tinyboards_db_views::{post_view::PostQuery, DeleteableOrRemoveable};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetFeed {
    type Response = ListPostsResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let params: &Self = &self;

        // get optional user view (don't need to be logged in to see posts)
        let user = load_user_opt(context.pool(), context.master_key(), auth).await?;

        // check if feed is visible or not
        check_private_instance(&user, context.pool()).await?;

        let listing_type = match params.listing_type.as_ref() {
            Some(ltype) => map_to_listing_type(Some(&ltype.to_lowercase())),
            None => ListingType::All,
        };

        let sort = match params.sort.as_ref() {
            Some(sort) => map_to_sort_type(Some(&sort.to_lowercase())),
            None => SortType::Hot,
        };

        let params_nsfw = params.is_nsfw;
        let board_id = params.board_id;
        let creator_id = params.creator_id;
        let search_term = params.search.clone();
        let saved_only = params.saved_only;
        let limit = params.limit;
        let page = params.page;
        let mut nsfw = false;
        let local_user_match = user.clone();

        // normally we would get if the user has show_nsfw set to true/false when querying posts
        if let Some(ref user) = user {
            nsfw = user.show_nsfw;
        };

        // if we are getting nsfw from query string param in the api call, override the user setting (allows querying of nsfw posts independent of auth)
        if params_nsfw.is_some() {
            nsfw = params_nsfw.unwrap();
        }

        let response = PostQuery::builder()
            .pool(context.pool())
            .listing_type(Some(listing_type))
            .sort(Some(sort))
            .board_id(board_id)
            .search_term(search_term)
            .local_user(local_user_match.as_ref())
            .person(user_match.as_ref())
            .creator_id(creator_id)
            .saved_only(saved_only)
            .show_nsfw(Some(nsfw))
            .limit(limit)
            .page(page)
            .build()
            .list()
            .await?;

        let mut posts = response.posts;
        let total_count = response.count;

        for pv in posts
            .iter_mut()
            .filter(|p| p.post.is_deleted || p.post.is_removed)
        {
            pv.hide_if_removed_or_deleted(user.as_ref());
        }

        Ok(ListPostsResponse { posts, total_count })
    }
}
