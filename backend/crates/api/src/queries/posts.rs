use crate::helpers::{permissions, validation::check_private_instance};
use crate::Censorable;
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        aggregates::PostAggregates,
        board::boards::Board as DbBoard,
        board::board_mods::BoardModerator,
        board::board_mods::ModPerms,
        post::posts::Post as DbPost,
        user::user::{AdminPerms, User as DbUser},
    },
    schema::{
        board_moderators, board_subscribers, boards, post_aggregates, post_hidden, post_saved,
        posts, users,
    },
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{structs::post::Post, ListingType, LoggedInUser, SortType};

#[derive(Default)]
pub struct QueryPosts;

#[Object]
impl QueryPosts {
    pub async fn post(&self, ctx: &Context<'_>, id: ID) -> Result<Post> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = permissions::optional_auth(ctx);

        check_private_instance(v_opt, pool).await?;

        let post_uuid: Uuid = id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid post ID".into()))?;

        let conn = &mut get_conn(pool).await?;

        let db_post: DbPost = posts::table
            .find(post_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

        // Check if board is banned (unless admin)
        let require_board_not_banned = match v_opt {
            Some(v) => !v.has_permission(AdminPerms::Boards),
            None => true,
        };

        if require_board_not_banned {
            let board: DbBoard = boards::table
                .find(db_post.board_id)
                .first(conn)
                .await
                .map_err(|_| TinyBoardsError::NotFound("Board not found".into()))?;
            if board.is_banned {
                return Err(TinyBoardsError::from_message(
                    403,
                    board
                        .public_ban_reason
                        .as_deref()
                        .unwrap_or("This board has been banned"),
                )
                .into());
            }
        }

        let agg: PostAggregates = post_aggregates::table
            .filter(post_aggregates::post_id.eq(post_uuid))
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post aggregates not found".into()))?;

        let mut post = Post::from((db_post, agg));

        let is_admin = v_opt
            .map(|v| v.has_permission(AdminPerms::Content))
            .unwrap_or(false);
        let is_mod = match v_opt {
            Some(v) => board_moderators::table
                .filter(board_moderators::board_id.eq(post.uuid_board_id))
                .filter(board_moderators::user_id.eq(v.id))
                .first::<BoardModerator>(conn)
                .await
                .ok()
                .map(|m| m.has_permission(ModPerms::Content))
                .unwrap_or(false),
            None => false,
        };

        let my_user_id = v_opt.map(|v| v.id).unwrap_or(Uuid::nil());
        if !is_admin {
            post.censor(my_user_id, is_admin, is_mod);
        }

        Ok(post)
    }

    pub async fn list_posts<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Limit of how many posts to load. Max value and default is 25.")]
        limit: Option<i64>,
        #[graphql(desc = "Sorting type.")] sort: Option<SortType>,
        #[graphql(desc = "Listing type, eg. \"Local\" or \"Subscribed\".")] listing_type: Option<
            ListingType,
        >,
        #[graphql(desc = "If specified, only posts from the given user will be loaded.")]
        _user_id: Option<ID>,
        #[graphql(desc = "If specified, only posts in the given board will be loaded.")]
        board_id: Option<ID>,
        user_name: Option<String>,
        board_name: Option<String>,
        #[graphql(desc = "Whether to only show saved posts.")] saved_only: Option<bool>,
        #[graphql(desc = "Whether to only show removed posts (admin/mod only).")] removed_only: Option<bool>,
        #[graphql(desc = "Whether to include removed posts (admin/mod only).")] include_removed: Option<bool>,
        #[graphql(desc = "Page.")] page: Option<i64>,
    ) -> Result<Vec<Post>> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = permissions::optional_auth(ctx);

        check_private_instance(v_opt, pool).await?;

        let sort = sort.unwrap_or(SortType::NewComments);
        let listing_type = listing_type.unwrap_or(ListingType::Local);
        let limit = std::cmp::min(limit.unwrap_or(25), 25);
        let page = page.unwrap_or(1);
        let offset = (page - 1) * limit;

        let is_admin = v_opt
            .map(|v| v.has_permission(AdminPerms::Content))
            .unwrap_or(false);

        let removed_only = removed_only.unwrap_or(false);
        let include_removed = include_removed.unwrap_or(false);

        let conn = &mut get_conn(pool).await?;

        // Resolve board_id from board_name if needed
        let board_uuid: Option<Uuid> = match board_name {
            Some(name) => {
                let board: Option<DbBoard> = boards::table
                    .filter(boards::name.eq(&name))
                    .first(conn)
                    .await
                    .optional()
                    .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
                board.map(|b| b.id)
            }
            None => match board_id {
                Some(bid) => Some(
                    bid.parse::<Uuid>()
                        .map_err(|_| TinyBoardsError::from_message(400, "Invalid board ID"))?,
                ),
                None => None,
            },
        };

        // Resolve user_id from user_name
        let user_uuid: Option<Uuid> = match user_name {
            Some(name) => {
                let user: Option<DbUser> = users::table
                    .filter(users::name.eq(&name))
                    .first(conn)
                    .await
                    .optional()
                    .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
                user.map(|u| u.id)
            }
            None => None,
        };

        // Permission check for removed content
        if (removed_only || include_removed) && !is_admin {
            if let Some(bid) = board_uuid {
                if let Some(v) = v_opt {
                    let is_mod = board_moderators::table
                        .filter(board_moderators::board_id.eq(bid))
                        .filter(board_moderators::user_id.eq(v.id))
                        .first::<BoardModerator>(conn)
                        .await
                        .ok()
                        .map(|m| m.has_permission(ModPerms::Content))
                        .unwrap_or(false);
                    if !is_mod {
                        return Err(TinyBoardsError::from_message(
                            403,
                            "Permission denied: cannot view removed content",
                        )
                        .into());
                    }
                } else {
                    return Err(TinyBoardsError::from_message(
                        403,
                        "Permission denied: cannot view removed content",
                    )
                    .into());
                }
            } else {
                return Err(TinyBoardsError::from_message(
                    403,
                    "Permission denied: cannot view removed content",
                )
                .into());
            }
        }

        // For saved_only, require auth
        if saved_only.unwrap_or(false) && v_opt.is_none() {
            return Err(
                TinyBoardsError::from_message(401, "Login required to view saved posts").into(),
            );
        }

        // Build the query
        let mut query = posts::table
            .inner_join(post_aggregates::table.on(post_aggregates::post_id.eq(posts::id)))
            .into_boxed();

        // Basic filters
        query = query.filter(posts::deleted_at.is_null());

        if !include_removed && !removed_only {
            query = query.filter(posts::is_removed.eq(false));
        }

        if let Some(bid) = board_uuid {
            query = query.filter(posts::board_id.eq(bid));
        }

        if let Some(uid) = user_uuid {
            query = query.filter(posts::creator_id.eq(uid));
        }

        // Listing type filters
        match listing_type {
            ListingType::Subscribed => {
                if let Some(v) = v_opt {
                    let subscribed_board_ids: Vec<Uuid> = board_subscribers::table
                        .filter(board_subscribers::user_id.eq(v.id))
                        .select(board_subscribers::board_id)
                        .load(conn)
                        .await
                        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
                    query = query.filter(posts::board_id.eq_any(subscribed_board_ids));
                }
            }
            ListingType::Moderated => {
                if let Some(v) = v_opt {
                    let moderated_board_ids: Vec<Uuid> = board_moderators::table
                        .filter(board_moderators::user_id.eq(v.id))
                        .select(board_moderators::board_id)
                        .load(conn)
                        .await
                        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
                    query = query.filter(posts::board_id.eq_any(moderated_board_ids));
                }
            }
            _ => {}
        }

        // Saved only filter
        if saved_only.unwrap_or(false) {
            if let Some(v) = v_opt {
                let saved_post_ids: Vec<Uuid> = post_saved::table
                    .filter(post_saved::user_id.eq(v.id))
                    .select(post_saved::post_id)
                    .load(conn)
                    .await
                    .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
                query = query.filter(posts::id.eq_any(saved_post_ids));
            }
        }

        // Exclude hidden posts for logged-in user
        if let Some(v) = v_opt {
            let hidden_post_ids: Vec<Uuid> = post_hidden::table
                .filter(post_hidden::user_id.eq(v.id))
                .select(post_hidden::post_id)
                .load(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
            if !hidden_post_ids.is_empty() {
                query = query.filter(posts::id.ne_all(hidden_post_ids));
            }
        }

        // Exclude banned boards unless admin
        if !is_admin {
            query = query.filter(
                posts::board_id.ne_all(
                    boards::table
                        .filter(boards::is_banned.eq(true))
                        .select(boards::id),
                ),
            );
        }

        if removed_only {
            query = query.filter(posts::is_removed.eq(true));
        }

        // Sort — featured/pinned posts always appear first.
        // Board view: prioritize board-level pins. Home feed: prioritize site-wide pins.
        if board_uuid.is_some() {
            query = query.order(post_aggregates::is_featured_board.desc());
        } else {
            query = query.order(post_aggregates::is_featured_local.desc());
        }

        query = match sort {
            SortType::New => query.then_order_by(posts::created_at.desc()),
            SortType::Old => query.then_order_by(posts::created_at.asc()),
            SortType::Hot => query.then_order_by(post_aggregates::hot_rank.desc()),
            SortType::Active => query.then_order_by(post_aggregates::hot_rank_active.desc()),
            SortType::TopDay | SortType::TopWeek | SortType::TopMonth | SortType::TopYear | SortType::TopAll => {
                query.then_order_by(post_aggregates::score.desc())
            }
            SortType::MostComments => query.then_order_by(post_aggregates::comments.desc()),
            SortType::NewComments => query.then_order_by(post_aggregates::newest_comment_time.desc()),
            SortType::Controversial => query.then_order_by(post_aggregates::controversy_rank.desc()),
        };

        // Time filter for top sorts
        match sort {
            SortType::TopDay => {
                query = query.filter(posts::created_at.gt(chrono::Utc::now() - chrono::Duration::days(1)));
            }
            SortType::TopWeek => {
                query = query.filter(posts::created_at.gt(chrono::Utc::now() - chrono::Duration::weeks(1)));
            }
            SortType::TopMonth => {
                query = query.filter(posts::created_at.gt(chrono::Utc::now() - chrono::Duration::days(30)));
            }
            SortType::TopYear => {
                query = query.filter(posts::created_at.gt(chrono::Utc::now() - chrono::Duration::days(365)));
            }
            _ => {}
        }

        query = query.limit(limit).offset(offset);

        let results: Vec<(DbPost, PostAggregates)> = query
            .select((posts::all_columns, post_aggregates::all_columns))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Post::from).collect())
    }

    /// Get user's hidden posts
    pub async fn get_hidden_posts(
        &self,
        ctx: &Context<'_>,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<Post>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        let page = page.unwrap_or(1) as i64;
        let limit = limit.unwrap_or(25).min(100) as i64;
        let offset = (page - 1) * limit;

        let conn = &mut get_conn(pool).await?;

        let results: Vec<(DbPost, PostAggregates)> = post_hidden::table
            .inner_join(posts::table.on(post_hidden::post_id.eq(posts::id)))
            .inner_join(
                post_aggregates::table.on(post_aggregates::post_id.eq(posts::id)),
            )
            .filter(post_hidden::user_id.eq(user.id))
            .select((posts::all_columns, post_aggregates::all_columns))
            .order(post_hidden::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Post::from).collect())
    }

    /// List posts for a specific board sorted by activity (newest comment time)
    pub async fn list_threads(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Board ID to fetch threads from")] board_id: ID,
        #[graphql(desc = "Limit of threads to load. Default is 25.")] limit: Option<i64>,
        #[graphql(desc = "Page number for pagination")] page: Option<i64>,
    ) -> Result<Vec<Post>> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = permissions::optional_auth(ctx);

        check_private_instance(v_opt, pool).await?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid board ID"))?;

        let conn = &mut get_conn(pool).await?;

        let board: DbBoard = boards::table
            .find(board_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found".into()))?;

        let require_board_not_banned = match v_opt {
            Some(v) => !v.has_permission(AdminPerms::Boards),
            None => true,
        };

        if require_board_not_banned && board.deleted_at.is_some() {
            return Err(TinyBoardsError::from_message(
                410,
                &format!("/b/{} is deleted.", &board.name),
            )
            .into());
        }

        if board.is_banned {
            let reason = board
                .public_ban_reason
                .as_deref()
                .unwrap_or("This board has been banned");
            return Err(TinyBoardsError::from_message(403, reason).into());
        }

        if board.mode != tinyboards_db::enums::DbBoardMode::Forum {
            return Err(TinyBoardsError::from_message(
                400,
                "This board is not a Forum board and does not support threads",
            )
            .into());
        }

        let limit = limit.unwrap_or(25).min(50);
        let page = page.unwrap_or(1);
        let offset = (page - 1) * limit;

        let results: Vec<(DbPost, PostAggregates)> = posts::table
            .inner_join(post_aggregates::table.on(post_aggregates::post_id.eq(posts::id)))
            .filter(posts::board_id.eq(board_uuid))
            .filter(posts::is_thread.eq(true))
            .filter(posts::deleted_at.is_null())
            .filter(posts::is_removed.eq(false))
            .order((
                post_aggregates::is_featured_board.desc(),
                post_aggregates::newest_comment_time.desc(),
            ))
            .select((posts::all_columns, post_aggregates::all_columns))
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Post::from).collect())
    }
}
