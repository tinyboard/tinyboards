use crate::helpers::validation::check_private_instance;
use async_graphql::*;
use tinyboards_db::{
    models::{board::boards::Board as DbBoard, person::local_user::AdminPerms},
    utils::DbPool,
};
use tinyboards_db_views::board_view::BoardQuery;
use tinyboards_utils::TinyBoardsError;

use crate::{structs::boards::Board, ListingType, LoggedInUser, SortType};

#[derive(Default)]
pub struct QueryBoards;

#[Object]
impl QueryBoards {
    pub async fn board(&self, ctx: &Context<'_>, name: String) -> Result<Board> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = ctx.data::<LoggedInUser>()?.inner();

        check_private_instance(v_opt, pool).await?;

        if name.contains("@") {
            todo!("Add apub support here");
        }

        DbBoard::get_with_counts_for_name(pool, name)
            .await
            .map(Board::from)
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load board.").into()
            })
    }

    pub async fn list_boards<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Limit of how many boards to load. Max value and default is 25.")]
        limit: Option<i64>,
        #[graphql(desc = "Page of the search. Default is zero.")] page: Option<i64>,
        #[graphql(desc = "Sorting type.")] sort: Option<SortType>,
        #[graphql(desc = "Listing type, eg. \"Local\" or \"Subscribed\".")] listing_type: Option<
            ListingType,
        >,
        #[graphql(desc = "Search term for board (fuzzy search for board name)")]
        search_term: Option<String>,
        #[graphql(desc = "Whether boards' title and description should also be searched")]
        search_title_and_desc: Option<bool>,
        #[graphql(desc = "Whether to list banned boards. Will do nothing if you're not an admin.")]
        banned_boards: Option<bool>,
    ) -> Result<Vec<Board>> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = ctx.data::<LoggedInUser>()?.inner();

        check_private_instance(v_opt, pool).await?;

        let sort = sort.unwrap_or(SortType::Hot);
        let listing_type = listing_type.unwrap_or(ListingType::Local);
        let limit = Some(std::cmp::min(limit.unwrap_or(25), 25));
        let search_title_and_desc = search_title_and_desc.unwrap_or(false);
        let person_id_join = match v_opt {
            Some(v) => v.person.id,
            None => -1,
        };
        let banned_boards = match v_opt {
            Some(v) => {
                v.local_user.has_permission(AdminPerms::Boards) && banned_boards.unwrap_or(false)
            }
            None => false,
        };

        let resp = DbBoard::list_with_counts(
            pool,
            person_id_join,
            limit,
            page,
            sort.into(),
            listing_type.into(),
            search_term,
            search_title_and_desc,
            banned_boards,
        )
        .await?;

        Ok(resp.into_iter().map(Board::from).collect::<Vec<Board>>())
    }
}
