use crate::{api::PerformApub, fetcher::resolve_actor_identifier, objects::board::ApubBoard};
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{Search, SearchResponse},
    utils::{check_private_instance, load_user_opt},
};
use tinyboards_db::{
    map_to_listing_type, map_to_search_type, map_to_sort_type,
    models::{board::boards::Board, site::local_site::LocalSite},
    utils::post_to_comment_sort_type,
    SearchType,
};
use tinyboards_db_views::{
    board_view::BoardQuery, comment_view::CommentQuery, person_view::PersonQuery,
    post_view::PostQuery,
};
use tinyboards_federation::config::Data;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait]
impl PerformApub for Search {
    type Response = SearchResponse;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        &self,
        context: &Data<TinyBoardsContext>,
        auth: Option<&str>,
    ) -> Result<SearchResponse, TinyBoardsError> {
        let data: &Search = self;

        let view = load_user_opt(context.pool(), context.master_key(), auth).await?;

        let _local_site = LocalSite::read(context.pool()).await?;

        check_private_instance(&view, context.pool()).await?;

        let mut posts = Vec::new();
        let mut comments = Vec::new();
        let mut boards = Vec::new();
        let mut users = Vec::new();

        let q = data.query.clone();
        let page = data.page;
        let limit = data.limit;

        let sort = data.sort.clone().map(|x| x.to_lowercase());
        let sort = map_to_sort_type(match sort {
            Some(ref sort) => sort,
            None => "hot",
        });

        let listing_type = map_to_listing_type(
            data.listing_type
                .clone()
                .map(|l| l.to_lowercase())
                .as_deref(),
        );
        let search_type =
            map_to_search_type(data.kind.clone().map(|s| s.to_lowercase()).as_deref());
        let board_id = if let Some(name) = &data.board_name {
            resolve_actor_identifier::<ApubBoard, Board>(name, context, &view.clone(), false)
                .await
                .ok()
                .map(|c| c.id)
        } else {
            data.board_id
        };
        let creator_id = data.creator_id;

        let local_user = view.map(|l| l.local_user);

        // I don't know why but the compiler complains when it is named `total_count`
        let mut _total_count = 0;

        match search_type {
            SearchType::Post => {
                let resp = PostQuery::builder()
                    .pool(context.pool())
                    .sort(Some(sort))
                    .listing_type(Some(listing_type))
                    .board_id(board_id)
                    .creator_id(creator_id)
                    .user(local_user.as_ref())
                    .search_term(q)
                    .page(page)
                    .limit(limit)
                    .build()
                    .list()
                    .await?;
                posts = resp.posts;
                _total_count = resp.count;
            }
            SearchType::Comment => {
                let resp = CommentQuery::builder()
                    .pool(context.pool())
                    .sort(Some(sort).map(post_to_comment_sort_type))
                    .listing_type(Some(listing_type))
                    .search_term(q)
                    .board_id(board_id)
                    .creator_id(creator_id)
                    .page(page)
                    .limit(limit)
                    .build()
                    .list()
                    .await?;
                comments = resp.comments;
                _total_count = resp.count;
            }
            SearchType::Person => {
                let resp = PersonQuery::builder()
                    .pool(context.pool())
                    .search_term(q)
                    .page(page)
                    .limit(limit)
                    .build()
                    .list()
                    .await?;
                users = resp.persons;
                _total_count = resp.count;
            }
            SearchType::Board => {
                let resp = BoardQuery::builder()
                    .pool(context.pool())
                    .sort(Some(sort))
                    .listing_type(Some(listing_type))
                    .search_term(q)
                    .user(local_user.as_ref())
                    .page(page)
                    .limit(limit)
                    .build()
                    .list()
                    .await?;
                boards = resp.boards;
                _total_count = resp.count;
            }
        }

        Ok(SearchResponse {
            kind: search_type.to_string(),
            comments,
            posts,
            boards,
            users,
            total_count: _total_count,
        })
    }
}
