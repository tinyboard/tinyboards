use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        aggregates::{BoardAggregates, CommentAggregates, PostAggregates, UserAggregates},
        board::boards::Board as DbBoard,
        comment::comments::Comment as DbComment,
        post::posts::Post as DbPost,
        user::user::User as DbUser,
    },
    schema::{
        board_aggregates, boards, comment_aggregates, comments, post_aggregates, posts,
        user_aggregates, users,
    },
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{
    structs::{
        boards::Board as GqlBoard,
        comment::Comment as GqlComment,
        post::Post as GqlPost,
        user::User as GqlUser,
    },
    LoggedInUser, SortType,
};

#[derive(Default)]
pub struct QuerySearch;

#[derive(SimpleObject)]
pub struct SearchResult {
    pub posts: Vec<GqlPost>,
    pub comments: Vec<GqlComment>,
    pub users: Vec<GqlUser>,
    pub boards: Vec<GqlBoard>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum SearchType {
    #[graphql(name = "all")]
    All,
    #[graphql(name = "posts")]
    Posts,
    #[graphql(name = "comments")]
    Comments,
    #[graphql(name = "users")]
    Users,
    #[graphql(name = "boards")]
    Boards,
}

#[Object]
impl QuerySearch {
    /// Search for content across posts, comments, users, and boards.
    /// Uses ILIKE for fuzzy matching (pg_trgm indexes exist for username and board name).
    // TODO: Add full-text search with tsvector/tsquery for better post/comment body search
    pub async fn search_content(
        &self,
        ctx: &Context<'_>,
        q: String,
        search_type: Option<SearchType>,
        sort: Option<SortType>,
        board_id: Option<ID>,
        creator_id: Option<ID>,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> Result<SearchResult> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let _user = ctx.data_unchecked::<LoggedInUser>();

        if q.trim().len() < 2 {
            return Err(TinyBoardsError::from_message(
                400,
                "Search query must be at least 2 characters",
            )
            .into());
        }

        let search_type = search_type.unwrap_or(SearchType::All);
        let _sort = sort.unwrap_or(SortType::New);
        let page = page.unwrap_or(1) as i64;
        let limit = limit.unwrap_or(20).min(50) as i64;
        let offset = (page - 1) * limit;

        let board_uuid: Option<Uuid> = match board_id {
            Some(bid) => Some(
                bid.parse::<Uuid>()
                    .map_err(|_| TinyBoardsError::from_message(400, "Invalid board ID"))?,
            ),
            None => None,
        };

        let creator_uuid: Option<Uuid> = match creator_id {
            Some(cid) => Some(
                cid.parse::<Uuid>()
                    .map_err(|_| TinyBoardsError::from_message(400, "Invalid creator ID"))?,
            ),
            None => None,
        };

        let search_term = format!("%{}%", q.to_lowercase());
        let mut result_posts = Vec::new();
        let mut result_comments = Vec::new();
        let mut result_users = Vec::new();
        let mut result_boards = Vec::new();

        // Search posts
        if matches!(search_type, SearchType::All | SearchType::Posts) {
            let mut query = posts::table
                .inner_join(post_aggregates::table.on(post_aggregates::post_id.eq(posts::id)))
                .into_boxed();

            query = query.filter(
                posts::title
                    .ilike(&search_term)
                    .or(posts::body.ilike(&search_term)),
            );

            if let Some(bid) = board_uuid {
                query = query.filter(posts::board_id.eq(bid));
            }
            if let Some(cid) = creator_uuid {
                query = query.filter(posts::creator_id.eq(cid));
            }

            query = query
                .filter(posts::is_removed.eq(false))
                .filter(posts::deleted_at.is_null());

            let post_results: Vec<(DbPost, PostAggregates)> = query
                .order(posts::created_at.desc())
                .select((posts::all_columns, post_aggregates::all_columns))
                .limit(limit)
                .offset(offset)
                .load(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            result_posts = post_results.into_iter().map(GqlPost::from).collect();
        }

        // Search comments
        if matches!(search_type, SearchType::All | SearchType::Comments) {
            let mut query = comments::table
                .inner_join(
                    comment_aggregates::table
                        .on(comment_aggregates::comment_id.eq(comments::id)),
                )
                .into_boxed();

            query = query.filter(comments::body.ilike(&search_term));

            if let Some(cid) = creator_uuid {
                query = query.filter(comments::creator_id.eq(cid));
            }
            if let Some(bid) = board_uuid {
                query = query.filter(comments::board_id.eq(bid));
            }

            query = query
                .filter(comments::is_removed.eq(false))
                .filter(comments::deleted_at.is_null());

            let comment_results: Vec<(DbComment, CommentAggregates)> = query
                .order(comments::created_at.desc())
                .select((comments::all_columns, comment_aggregates::all_columns))
                .limit(limit)
                .offset(offset)
                .load(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            result_comments = comment_results.into_iter().map(GqlComment::from).collect();
        }

        // Search users
        if matches!(search_type, SearchType::All | SearchType::Users) {
            let user_results: Vec<(DbUser, Option<UserAggregates>)> = users::table
                .left_join(user_aggregates::table.on(user_aggregates::user_id.eq(users::id)))
                .filter(
                    users::name
                        .ilike(&search_term)
                        .or(users::display_name.ilike(&search_term)),
                )
                .filter(users::is_banned.eq(false))
                .filter(users::deleted_at.is_null())
                .order(users::created_at.desc())
                .select((users::all_columns, user_aggregates::all_columns.nullable()))
                .limit(limit)
                .offset(offset)
                .load(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            for (user, agg) in user_results {
                result_users.push(GqlUser::from_db(user, agg));
            }
        }

        // Search boards
        if matches!(search_type, SearchType::All | SearchType::Boards) {
            let board_results: Vec<(DbBoard, Option<BoardAggregates>)> = boards::table
                .left_join(
                    board_aggregates::table.on(board_aggregates::board_id.eq(boards::id)),
                )
                .filter(
                    boards::name
                        .ilike(&search_term)
                        .or(boards::title.ilike(&search_term))
                        .or(boards::description.ilike(&search_term)),
                )
                .filter(boards::is_banned.eq(false))
                .filter(boards::deleted_at.is_null())
                .order(boards::created_at.desc())
                .select((boards::all_columns, board_aggregates::all_columns.nullable()))
                .limit(limit)
                .offset(offset)
                .load(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            result_boards = board_results
                .into_iter()
                .map(|(board, agg)| GqlBoard::from_db(board, agg))
                .collect();
        }

        Ok(SearchResult {
            posts: result_posts,
            comments: result_comments,
            users: result_users,
            boards: result_boards,
        })
    }
}
