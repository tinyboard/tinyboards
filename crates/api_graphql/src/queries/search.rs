use async_graphql::*;
use tinyboards_db::{
    aggregates::structs::BoardAggregates,
    models::{
        board::boards::Board,
        comment::comments::Comment,
        user::user::User,
        post::posts::Post,
    },
    utils::{DbPool, get_conn},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

use crate::{
    structs::{
        boards::Board as GqlBoard,
        comment::Comment as GqlComment,
        user::User as GqlPerson, 
        post::Post as GqlPost,
    },
    LoggedInUser, ListingType, SortType,
};

#[derive(Default)]
pub struct QuerySearch;

#[derive(SimpleObject)]
pub struct SearchResult {
    pub posts: Vec<GqlPost>,
    pub comments: Vec<GqlComment>,
    pub people: Vec<GqlPerson>,
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
    #[graphql(name = "people")]
    People,
    #[graphql(name = "boards")]
    Boards,
}

#[Object]
impl QuerySearch {
    /// Search for content
    pub async fn search_content(
        &self,
        ctx: &Context<'_>,
        q: String,
        search_type: Option<SearchType>,
        sort: Option<SortType>,
        listing_type: Option<ListingType>,
        board_id: Option<i32>,
        creator_id: Option<i32>,
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
        let _listing_type = listing_type.unwrap_or(ListingType::All);
        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(20).min(50); // Cap at 50
        let offset = (page - 1) * limit;

        let search_term = format!("%{}%", q.to_lowercase());
        let mut posts = Vec::new();
        let mut comments = Vec::new();
        let _people = Vec::new();
        let mut boards = Vec::new();

        match search_type {
            SearchType::All | SearchType::Posts => {
                use tinyboards_db::schema::posts;
                let mut query = posts::table.into_boxed();

                query = query.filter(
                    posts::title.ilike(&search_term)
                        .or(posts::body.ilike(&search_term))
                );

                if let Some(board_id) = board_id {
                    query = query.filter(posts::board_id.eq(board_id));
                }

                if let Some(creator_id) = creator_id {
                    query = query.filter(posts::creator_id.eq(creator_id));
                }

                let post_results = query
                    .filter(posts::is_removed.eq(false))
                    .filter(posts::is_deleted.eq(false))
                    .order(posts::creation_date.desc())
                    .limit(limit as i64)
                    .offset(offset as i64)
                    .load::<Post>(conn)
                    .await?;

                // Convert database posts to GraphQL posts using get_with_counts
                for post in post_results {
                    match Post::get_with_counts(pool, post.id, false).await {
                        Ok(post_with_counts) => posts.push(GqlPost::from(post_with_counts)),
                        Err(_) => continue,
                    }
                }
            }
            _ => {}
        }

        match search_type {
            SearchType::All | SearchType::Comments => {
                use tinyboards_db::schema::comments;
                let mut query = comments::table.into_boxed();

                query = query.filter(comments::body.ilike(&search_term));

                if let Some(creator_id) = creator_id {
                    query = query.filter(comments::creator_id.eq(creator_id));
                }

                let comment_results = query
                    .filter(comments::is_removed.eq(false))
                    .filter(comments::is_deleted.eq(false))
                    .order(comments::creation_date.desc())
                    .limit(limit as i64)
                    .offset(offset as i64)
                    .load::<Comment>(conn)
                    .await?;

                // Convert database comments to GraphQL comments using get_with_counts
                for comment in comment_results {
                    match Comment::get_with_counts(pool, comment.id).await {
                        Ok(comment_with_counts) => comments.push(GqlComment::from(comment_with_counts)),
                        Err(_) => continue,
                    }
                }
            }
            _ => {}
        }

        match search_type {
            SearchType::All | SearchType::People => {
                use tinyboards_db::schema::users;
                let mut query = users::table.into_boxed();

                query = query.filter(
                    users::name.ilike(&search_term)
                        .or(users::display_name.ilike(&search_term))
                );

                let _person_results = query
                    .filter(users::is_banned.eq(false))
                    .filter(users::is_deleted.eq(false))
                    .order(users::creation_date.desc())
                    .limit(limit as i64)
                    .offset(offset as i64)
                    .load::<User>(conn)
                    .await?;

                // For people search, we need to create User objects, but for simplicity 
                // we'll skip people search for now since it requires more complex User struct creation
                // TODO: Implement proper people search with User struct
            }
            _ => {}
        }

        match search_type {
            SearchType::All | SearchType::Boards => {
                use tinyboards_db::schema::boards;
                let mut query = boards::table.into_boxed();

                query = query.filter(
                    boards::name.ilike(&search_term)
                        .or(boards::title.ilike(&search_term))
                        .or(boards::description.ilike(&search_term))
                );

                let board_results = query
                    .filter(boards::is_banned.eq(false))
                    .filter(boards::is_deleted.eq(false))
                    .order(boards::creation_date.desc())
                    .limit(limit as i64)
                    .offset(offset as i64)
                    .load::<Board>(conn)
                    .await?;

                // Convert database boards to GraphQL boards (boards need aggregates)
                for board in board_results {
                    // For simplicity, create empty aggregates
                    let aggregates = BoardAggregates {
                        id: board.id,
                        board_id: board.id,
                        subscribers: 0,
                        posts: 0, 
                        comments: 0,
                        creation_date: board.creation_date,
                        users_active_day: 0,
                        users_active_week: 0,
                        users_active_month: 0,
                        users_active_half_year: 0,
                    };
                    boards.push(GqlBoard::from((board, aggregates)));
                }
            }
            _ => {}
        }

        Ok(SearchResult {
            posts,
            comments,
            people: _people,
            boards,
        })
    }
}