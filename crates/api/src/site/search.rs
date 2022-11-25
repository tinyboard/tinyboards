use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{Search, SearchResponse},
    utils::{blocking, check_private_instance, get_user_view_from_jwt_opt},
};
use tinyboards_db::{
    map_to_listing_type,
    map_to_sort_type,
    map_to_search_type,
    ListingType,
    SortType,
    SearchType,
    utils::post_to_comment_sort_type,
};
use tinyboards_db_views::{
    post_view::PostQuery,
    comment_view::CommentQuery,
    user_view::UserQuery,
    board_view::BoardQuery,
    DeleteableOrRemoveable,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for Search {
    type Response = SearchResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<SearchResponse, TinyBoardsError> {

        let params: &Self = &self;

        // get optional user view
        let user_view = 
            get_user_view_from_jwt_opt(auth, context.pool(), context.master_key()).await?;

        // search should not function on private instances if you are not authed
        check_private_instance(&user_view, context.pool()).await?;

        // get the search type
        let search_type = map_to_search_type(params.kind.as_deref());

        let user_id = user_view.as_ref().map(|u| u.user.id);
        let user = user_view.as_ref().map(|u| u.user.clone());
        
        let mut posts= Vec::new();
        let mut comments = Vec::new();
        let mut boards = Vec::new();
        let mut users = Vec::new();

        let search_term = params.query.clone();
        let url_search = params.domain.clone();
        let page = params.page.clone();
        let limit = params.limit.clone();

        let sort = match params.sort.as_ref() {
            Some(sort) => map_to_sort_type(Some(&sort.to_lowercase())),
            None => SortType::Hot,
        };

        let comment_sort_type = post_to_comment_sort_type(sort);

        let listing_type = match params.listing_type.as_ref() {
            Some(ltype) => map_to_listing_type(Some(&ltype.to_lowercase())),
            None => ListingType::All,
        };

        let board_id = params.board_id.clone();
        let creator_id = params.creator_id.clone();

        match search_type {
            SearchType::Post => {
                posts = blocking(context.pool(), move |conn| {
                    PostQuery::builder()
                        .conn(conn)
                        .sort(Some(sort))
                        .listing_type(Some(listing_type))
                        .board_id(board_id)
                        .creator_id(creator_id)
                        .user(user.as_ref())
                        .search_term(search_term)
                        .url_search(url_search)
                        .page(page)
                        .limit(limit)
                        .build()
                        .list()

                }).await??;
            },
            SearchType::Comment => {
                comments = blocking(context.pool(), move |conn| {
                    CommentQuery::builder()
                        .conn(conn)
                        .sort(Some(comment_sort_type))
                        .listing_type(Some(listing_type))
                        .search_term(search_term)
                        .board_id(board_id)
                        .creator_id(creator_id)
                        .user(user.as_ref())
                        .page(page)
                        .limit(limit)
                        .build()
                        .list()
                }).await??;
            },
            SearchType::Board => {
                boards = blocking(context.pool(), move |conn| {
                    BoardQuery::builder()
                        .conn(conn)
                        .listing_type(Some(listing_type))
                        .sort(Some(sort))
                        .user(user.as_ref())
                        .search_term(search_term)
                        .page(page)
                        .limit(limit)
                        .build()
                        .list()
                }).await??;
            },
            SearchType::User => {
                users = blocking(context.pool(), move |conn| {
                    UserQuery::builder()
                        .conn(conn)
                        .search_term(search_term)
                        .page(page)
                        .limit(limit)
                        .build()
                        .list()
                }).await??;
            },
        };

        if user_id.is_none() {
            // TODO: blank out info for deleted or removed boards here too!

            // hide info if comment is deleted or removed
            for cv in comments
            .iter_mut()
            {
                cv.hide_if_removed_or_deleted(user_view.as_ref());
            }

            // hide info if post is deleted or removed
            for pv in posts
            .iter_mut()
            {
                pv.hide_if_removed_or_deleted(user_view.as_ref());
            }
        }


        Ok(SearchResponse { 
            kind: search_type.to_string(), 
            comments, 
            posts, 
            boards, 
            users, 
        })

    }

}