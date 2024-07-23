use async_graphql::*;
use tinyboards_db::{models::board::boards::Board as DbBoard, utils::DbPool};
use tinyboards_db_views::board_view::BoardQuery;
use tinyboards_utils::TinyBoardsError;

use crate::{
    structs::boards::Board,
    LoggedInUser,
    ListingType,
    SortType,
};

#[derive(Default)]
pub struct QueryBoards;

#[Object]
impl QueryBoards {
    pub async fn list_boards<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc="Limit of how many boards to load. Max value and default is 25.")]
        limit: Option<i64>,
        #[graphql(desc="Page of the search. Default is zero.")]
        page: Option<i64>,
        #[graphql(desc="Sorting type.")]
        sort: Option<SortType>,
        #[graphql(desc="Listing type, eg. \"Local\" or \"Subscribed\".")]
        listing_type: Option<ListingType>,
        #[graphql(desc="Search term for board (fuzzy search for board name)")]
        search_term: Option<String>,

    ) -> Result<Vec<Board>> {
        
        let pool = ctx.data::<DbPool>()?;
        let v_opt = ctx.data::<LoggedInUser>()?.inner();
        let user_opt = v_opt.map(|u| &u.local_user);
        let sort = sort.unwrap_or(SortType::Active);
        let listing_type = listing_type.unwrap_or(ListingType::Local);
        let limit = std::cmp::min(limit.unwrap_or(25), 25);

        let resp = BoardQuery::builder()
            .pool(pool)
            .user(user_opt)
            .sort(Some(sort))
            .listing_type(Some(listing_type))
            .limit(Some(limit))
            .page(Some(page))
            .build()
            .list()
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Fetching boards failed.")
            })?;

        Ok(resp.into_iter().map(Board::from).collect::<Vec<Board>>())

    }
}