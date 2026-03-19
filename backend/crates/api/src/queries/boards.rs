use crate::helpers::{permissions, validation::check_private_instance};
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        aggregates::BoardAggregates as DbBoardAggregates,
        board::boards::Board as DbBoard,
        user::user::AdminPerms,
    },
    schema::{board_aggregates, board_subscribers, boards},
    utils::{get_conn, DbPool, fuzzy_search},
};
use tinyboards_utils::TinyBoardsError;

use crate::{structs::boards::Board, ListingType, SortType};

#[derive(Default)]
pub struct QueryBoards;

#[Object]
impl QueryBoards {
    /// Get a single board by name.
    pub async fn board(&self, ctx: &Context<'_>, name: String) -> Result<Board> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = permissions::optional_auth(ctx);

        check_private_instance(v_opt, pool).await?;

        if name.contains('@') {
            return Err(TinyBoardsError::from_message(501, "Federation not supported").into());
        }

        let conn = &mut get_conn(pool).await?;

        let db_board: DbBoard = boards::table
            .filter(boards::name.eq(&name))
            .filter(boards::deleted_at.is_null())
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound(format!("Board '{}' not found", name)))?;

        let agg: Option<DbBoardAggregates> = board_aggregates::table
            .filter(board_aggregates::board_id.eq(db_board.id))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Check if the current user is subscribed to this board
        let is_subscribed = if let Some(user) = v_opt {
            board_subscribers::table
                .filter(board_subscribers::board_id.eq(db_board.id))
                .filter(board_subscribers::user_id.eq(user.id))
                .first::<tinyboards_db::models::social::BoardSubscriber>(conn)
                .await
                .is_ok()
        } else {
            false
        };

        Ok(Board::from_db_with_sub(db_board, agg, is_subscribed))
    }

    /// List boards with optional filtering and pagination.
    pub async fn list_boards<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Limit of how many boards to load. Max value and default is 25.")]
        limit: Option<i64>,
        #[graphql(desc = "Page of the search. Default is 1.")]
        page: Option<i64>,
        #[graphql(desc = "Sorting type.")]
        sort: Option<SortType>,
        #[graphql(desc = "Listing type, eg. \"Local\" or \"Subscribed\".")]
        listing_type: Option<ListingType>,
        #[graphql(desc = "Search term for board name (fuzzy search).")]
        search_term: Option<String>,
        #[graphql(desc = "Whether boards' title and description should also be searched.")]
        search_title_and_desc: Option<bool>,
        #[graphql(desc = "Whether to list banned boards. Ignored if not an admin.")]
        banned_boards: Option<bool>,
    ) -> Result<Vec<Board>> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = permissions::optional_auth(ctx);

        check_private_instance(v_opt, pool).await?;

        let limit = std::cmp::min(limit.unwrap_or(25), 25);
        let offset = (page.unwrap_or(1) - 1) * limit;
        let search_title_and_desc = search_title_and_desc.unwrap_or(false);

        // Only admins with Boards permission may view banned boards.
        let show_banned = match v_opt {
            Some(v) => v.has_permission(AdminPerms::Boards) && banned_boards.unwrap_or(false),
            None => false,
        };

        let conn = &mut get_conn(pool).await?;

        let mut query = boards::table
            .left_join(board_aggregates::table.on(board_aggregates::board_id.eq(boards::id)))
            .filter(boards::deleted_at.is_null())
            .filter(boards::is_removed.eq(false))
            .into_boxed();

        if !show_banned {
            query = query.filter(boards::is_banned.eq(false));
        }

        if let Some(ref term) = search_term {
            if search_title_and_desc {
                query = query.filter(
                    boards::name.ilike(fuzzy_search(term))
                        .or(boards::title.ilike(fuzzy_search(term)))
                        .or(boards::description.ilike(fuzzy_search(term))),
                );
            } else {
                query = query.filter(boards::name.ilike(fuzzy_search(term)));
            }
        }

        // Apply sort. Aggregate columns default to 0 when no row exists.
        let sort = sort.unwrap_or(SortType::Hot);
        let listing_type = listing_type.unwrap_or(ListingType::Local);

        // Hidden boards are excluded from general listings unless the user is
        // subscribed or an admin.
        let is_admin = v_opt.map(|v| v.is_admin).unwrap_or(false);
        if !is_admin {
            query = query.filter(boards::is_hidden.eq(false));
        }

        // Subscribed listing filter
        let user_id_opt = v_opt.map(|v| v.id);
        if listing_type == ListingType::Subscribed {
            if let Some(uid) = user_id_opt {
                let subscribed_board_ids: Vec<uuid::Uuid> = board_subscribers::table
                    .filter(board_subscribers::user_id.eq(uid))
                    .select(board_subscribers::board_id)
                    .load(conn)
                    .await
                    .unwrap_or_default();
                query = query.filter(boards::id.eq_any(subscribed_board_ids));
            } else {
                // Not logged in, return empty
                return Ok(vec![]);
            }
        }

        let results: Vec<(DbBoard, Option<DbBoardAggregates>)> = match sort {
            SortType::New | SortType::Active => query
                .order(boards::created_at.desc())
                .limit(limit)
                .offset(offset)
                .select((boards::all_columns, board_aggregates::all_columns.nullable()))
                .load(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?,
            _ => query
                .order(
                    board_aggregates::subscribers
                        .nullable()
                        .desc()
                        .nulls_last(),
                )
                .limit(limit)
                .offset(offset)
                .select((boards::all_columns, board_aggregates::all_columns.nullable()))
                .load(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?,
        };

        // Check subscription status for each board
        let subscribed_board_ids: std::collections::HashSet<uuid::Uuid> = if let Some(uid) = user_id_opt {
            board_subscribers::table
                .filter(board_subscribers::user_id.eq(uid))
                .select(board_subscribers::board_id)
                .load::<uuid::Uuid>(conn)
                .await
                .unwrap_or_default()
                .into_iter()
                .collect()
        } else {
            std::collections::HashSet::new()
        };

        let boards_out = results
            .into_iter()
            .map(|(b, agg)| {
                let is_sub = subscribed_board_ids.contains(&b.id);
                Board::from_db_with_sub(b, agg, is_sub)
            })
            .collect();

        Ok(boards_out)
    }
}
