use crate::{
    api::PerformApub,
    fetcher::resolve_actor_identifier,
    objects::board::ApubBoard,
  };
  use tinyboards_federation::config::Data;
  use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{Search, SearchResponse},
    utils::{check_private_instance, require_user},
  };
  use tinyboards_db::{
    models::{board::boards::Board, site::local_site::LocalSite, person::local_user::LocalUser},
    utils::post_to_comment_sort_type,
    SearchType, map_to_search_type, map_to_listing_type, map_to_sort_type, traits::Crud,
  };
  use tinyboards_db_views::{comment_view::CommentQuery, post_view::PostQuery, board_view::BoardQuery, person_view::PersonQuery};
  use tinyboards_utils::error::TinyBoardsError;

  #[async_trait::async_trait]
  impl PerformApub for Search {
    type Response = SearchResponse;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(&self, context: &Data<TinyBoardsContext>, auth: Option<&str>) -> Result<SearchResponse, TinyBoardsError> {
        let data: &Search = self;

        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        let _local_site = LocalSite::read(context.pool()).await?;

        check_private_instance(&Some(view.local_user.clone()), context.pool()).await?;

        let mut posts = Vec::new();
        let mut comments = Vec::new();
        let mut boards = Vec::new();
        let mut users = Vec::new();

        let q = data.query.clone();
        let page = data.page;
        let limit = data.limit;
        let sort = map_to_sort_type(data.sort.clone().map(|s| s.to_lowercase()).as_deref());
        let listing_type = map_to_listing_type(data.listing_type.clone().map(|l| l.to_lowercase()).as_deref());
        let search_type = map_to_search_type(data.kind.clone().map(|s| s.to_lowercase()).as_deref());
        let board_id = if let Some(name) = &data.board_name {
            resolve_actor_identifier::<ApubBoard, Board>(name, context, &Some(view.clone()), false)
                .await
                .ok()
                .map(|c| c.id)
        } else {
            data.board_id
        };
        let creator_id = data.creator_id;

        let local_user = LocalUser::read(context.pool(), view.local_user.id.clone()).await?;

        match search_type {
            SearchType::Post => {
                posts = PostQuery::builder()
                    .pool(context.pool())
                    .sort(Some(sort))
                    .listing_type(Some(listing_type))
                    .board_id(board_id)
                    .creator_id(creator_id)
                    .user(Some(&local_user))
                    .search_term(q)
                    .page(page)
                    .limit(limit)
                    .build()
                    .list()
                    .await?
                    .posts;
            },
            SearchType::Comment => {
                comments = CommentQuery::builder()
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
                    .await?
                    .comments;
            },
            SearchType::Person => {
                users = PersonQuery::builder()
                    .pool(context.pool())
                    .search_term(q)
                    .page(page)
                    .limit(limit)
                    .build()
                    .list()
                    .await?
                    .persons;
            },
            SearchType::Board => {
                boards = BoardQuery::builder()
                    .pool(context.pool())
                    .sort(Some(sort))
                    .listing_type(Some(listing_type))
                    .search_term(q)
                    .user(Some(&view.local_user))
                    .page(page)
                    .limit(limit)
                    .build()
                    .list()
                    .await?
                    .boards;
            },
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