use crate::helpers::{permissions, validation::check_private_instance};
use crate::Censorable;
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        aggregates::CommentAggregates,
        board::board_mods::{BoardModerator, ModPerms},
        comment::comments::Comment as DbComment,
        user::user::{AdminPerms, User as DbUser},
    },
    schema::{board_moderators, boards, comment_aggregates, comments, users},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{structs::comment::Comment, LoggedInUser, CommentSortType};

#[derive(Default)]
pub struct QueryComments;

#[Object]
impl QueryComments {
    /// Get a single comment by ID
    pub async fn comment(&self, ctx: &Context<'_>, id: ID) -> Result<Comment> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = permissions::optional_auth(ctx);

        check_private_instance(v_opt, pool).await?;

        let comment_uuid: Uuid = id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid comment ID".into()))?;

        let conn = &mut get_conn(pool).await?;

        let db_comment: DbComment = comments::table
            .find(comment_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Comment not found".into()))?;

        let agg: CommentAggregates = comment_aggregates::table
            .filter(comment_aggregates::comment_id.eq(comment_uuid))
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Comment aggregates not found".into()))?;

        let mut comment = Comment::from((db_comment.clone(), agg));

        let is_admin = v_opt
            .map(|v| v.has_permission(AdminPerms::Content))
            .unwrap_or(false);
        let is_mod = match v_opt {
            Some(v) => board_moderators::table
                .filter(board_moderators::board_id.eq(db_comment.board_id))
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
            comment.censor(my_user_id, is_admin, is_mod);
        }

        Ok(comment)
    }

    /// List comments with filtering options
    pub async fn comments<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Limit of how many comments to load. Max value and default is 50.")]
        limit: Option<i32>,
        #[graphql(desc = "Page number for pagination.")] page: Option<i32>,
        #[graphql(desc = "Sorting type.")] sort: Option<CommentSortType>,
        #[graphql(desc = "If specified, only comments from the given post will be loaded.")]
        post_id: Option<ID>,
        #[graphql(desc = "If specified, only comments from the given user will be loaded.")]
        user_id: Option<ID>,
        #[graphql(desc = "If specified, only comments in the given board will be loaded.")]
        board_id: Option<ID>,
        #[graphql(desc = "Username to filter by.")] user_name: Option<String>,
        #[graphql(desc = "Board name to filter by.")] board_name: Option<String>,
        #[graphql(desc = "Whether to only show removed comments (admin/mod only).")]
        removed_only: Option<bool>,
        #[graphql(desc = "Whether to include removed comments (admin/mod only).")]
        include_removed: Option<bool>,
    ) -> Result<Vec<Comment>> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = permissions::optional_auth(ctx);

        check_private_instance(v_opt, pool).await?;

        let sort = sort.unwrap_or(CommentSortType::New);
        let limit = std::cmp::min(limit.unwrap_or(50), 100) as i64;
        let page = page.unwrap_or(1) as i64;
        let offset = (page - 1) * limit;

        let is_admin = v_opt
            .map(|v| v.has_permission(AdminPerms::Content))
            .unwrap_or(false);

        let removed_only = removed_only.unwrap_or(false);
        let include_removed = include_removed.unwrap_or(false);

        let conn = &mut get_conn(pool).await?;

        // Resolve board_id
        let board_uuid: Option<Uuid> = match board_name {
            Some(name) => boards::table
                .filter(boards::name.eq(&name))
                .select(boards::id)
                .first(conn)
                .await
                .optional()
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?,
            None => match board_id {
                Some(bid) => Some(
                    bid.parse::<Uuid>()
                        .map_err(|_| TinyBoardsError::from_message(400, "Invalid board ID"))?,
                ),
                None => None,
            },
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
            None => match user_id {
                Some(uid) => Some(
                    uid.parse::<Uuid>()
                        .map_err(|_| TinyBoardsError::from_message(400, "Invalid user ID"))?,
                ),
                None => None,
            },
        };

        // Resolve post_id
        let post_uuid: Option<Uuid> = match post_id {
            Some(pid) => Some(
                pid.parse::<Uuid>()
                    .map_err(|_| TinyBoardsError::from_message(400, "Invalid post ID"))?,
            ),
            None => None,
        };

        // Build query
        let mut query = comments::table
            .inner_join(
                comment_aggregates::table
                    .on(comment_aggregates::comment_id.eq(comments::id)),
            )
            .into_boxed();

        // Basic filters
        query = query.filter(comments::deleted_at.is_null());

        if !include_removed && !removed_only {
            query = query.filter(comments::is_removed.eq(false));
        }

        if removed_only {
            query = query.filter(comments::is_removed.eq(true));
        }

        if let Some(pid) = post_uuid {
            query = query.filter(comments::post_id.eq(pid));
        }

        if let Some(uid) = user_uuid {
            query = query.filter(comments::creator_id.eq(uid));
        }

        if let Some(bid) = board_uuid {
            query = query.filter(comments::board_id.eq(bid));
        }

        // Sort
        query = match sort {
            CommentSortType::New => query.order(comments::created_at.desc()),
            CommentSortType::Old => query.order(comments::created_at.asc()),
            CommentSortType::Top => query.order(comment_aggregates::score.desc()),
            CommentSortType::Hot => query.order(comment_aggregates::hot_rank.desc()),
        };

        query = query.limit(limit).offset(offset);

        let results: Vec<(DbComment, CommentAggregates)> = query
            .select((comments::all_columns, comment_aggregates::all_columns))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Comment::from).collect())
    }
}
