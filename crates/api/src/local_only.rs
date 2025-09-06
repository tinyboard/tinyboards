// Local-only implementations for operations that were previously federated

use crate::Perform;
use actix_web::web::Data;
use async_trait::async_trait;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{ResolveObject, ResolveObjectResponse, Search, SearchResponse},
    utils::get_user_from_header_opt,
};
use tinyboards_db_views::structs::{BoardView, CommentView, PersonView, PostView};
use tinyboards_utils::TinyBoardsError;

#[async_trait(?Send)]
impl Perform<'_> for Search {
    type Response = SearchResponse;
    type Route = ();

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _path: (),
        auth: Option<&str>,
    ) -> Result<SearchResponse, TinyBoardsError> {
        let data = self;
        let _local_user_view = get_user_from_header_opt(context.pool(), context.master_key(), auth).await;

        // Local search implementation - search only local content
        let posts = Vec::new();
        let comments = Vec::new();
        let boards = Vec::new();
        let users = Vec::new();

        // For now, return empty results for local-only search
        // TODO: Implement actual local search functionality
        if let Some(_query) = &data.query {
            // Local search would be implemented here
            // For now, just return empty results
        }

        Ok(SearchResponse {
            kind: "All".to_string(),
            posts,
            comments,
            boards,
            users,
            total_count: 0,
        })
    }
}

#[async_trait(?Send)]
impl Perform<'_> for ResolveObject {
    type Response = ResolveObjectResponse;
    type Route = ();

    async fn perform(
        self,
        _context: &Data<TinyBoardsContext>,
        _path: (),
        _auth: Option<&str>,
    ) -> Result<ResolveObjectResponse, TinyBoardsError> {
        // In local-only mode, we cannot resolve federated objects
        Err(TinyBoardsError::from_message(
            501,
            "Object resolution not supported in local-only mode",
        ))
    }
}