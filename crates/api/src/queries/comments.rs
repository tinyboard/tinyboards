use crate::helpers::validation::check_private_instance;
use crate::Censorable;
use async_graphql::*;
use tinyboards_db::models::board::board_mods::ModPerms;
use tinyboards_db::{
    models::{
        board::board_mods::BoardModerator as DbBoardMod,
        user::user::{AdminPerms, User as DbUser},
        comment::comments::Comment as DbComment,
    },
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

use crate::{structs::comment::Comment, LoggedInUser, CommentSortType};

#[derive(Default)]
pub struct QueryComments;

#[Object]
impl QueryComments {
    /// Get a single comment by ID
    pub async fn comment(&self, ctx: &Context<'_>, id: i32) -> Result<Comment> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = ctx.data::<LoggedInUser>()?.inner();

        check_private_instance(v_opt, pool).await?;

        let res = DbComment::get_with_counts(pool, id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Comment not found"))?;

        let comment_board_id = res.0.board_id;
        let mut comment = Comment::from(res);
        let is_admin = match v_opt {
            Some(v) => v.has_permission(AdminPerms::Content),
            None => false,
        };
        let is_mod = match v_opt {
            Some(v) => {
                let mod_rel =
                    DbBoardMod::get_by_user_id_for_board(pool, v.id, comment_board_id, true)
                        .await;
                match mod_rel {
                    Ok(m) => m.has_permission(ModPerms::Content),
                    Err(_) => false,
                }
            }
            None => false,
        };

        let my_user_id = v_opt.as_ref().map(|v| v.id).unwrap_or(-1);
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
        post_id: Option<i32>,
        #[graphql(desc = "If specified, only comments from the given user will be loaded.")]
        user_id: Option<i32>,
        #[graphql(desc = "If specified, only comments in the given board will be loaded.")]
        board_id: Option<i32>,
        #[graphql(desc = "Username to filter by.")] user_name: Option<String>,
        #[graphql(desc = "Board name to filter by.")] board_name: Option<String>,
        #[graphql(desc = "Whether to only show removed comments (admin/mod only).")]
        removed_only: Option<bool>,
        #[graphql(desc = "Whether to include removed comments (admin/mod only).")]
        include_removed: Option<bool>,
    ) -> Result<Vec<Comment>> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = ctx.data::<LoggedInUser>()?.inner();

        check_private_instance(v_opt, pool).await?;

        let sort = sort.unwrap_or(CommentSortType::New);
        let limit = std::cmp::min(limit.unwrap_or(50), 100);
        let page = page.unwrap_or(1);
        let offset = (page - 1) * limit;

        let user_id_join = match v_opt {
            Some(v) => v.id,
            None => -1,
        };

        // Check permissions for removed content access
        let is_admin = match v_opt {
            Some(v) => v.has_permission(AdminPerms::Content),
            None => false,
        };

        let removed_only = removed_only.unwrap_or(false);
        let include_removed = include_removed.unwrap_or(false);

        // Only admins/mods can view removed content
        if (removed_only || include_removed) && !is_admin {
            // Check if user is a moderator for the specified board
            if let Some(board_id) = board_id {
                let mod_rel = DbBoardMod::get_by_user_id_for_board(pool, user_id_join, board_id, true).await;
                let is_mod = match mod_rel {
                    Ok(m) => m.has_permission(ModPerms::Content),
                    Err(_) => false,
                };
                if !is_mod {
                    return Err(TinyBoardsError::from_message(403, "Permission denied: cannot view removed content").into());
                }
            } else {
                return Err(TinyBoardsError::from_message(403, "Permission denied: cannot view removed content").into());
            }
        }

        // Resolve user_name to user_id
        let user_id = match user_name {
            Some(name) => DbUser::get_by_name(pool, name)
                .await
                .map(|u| Some(u.id))
                .unwrap_or(Some(0)),
            None => user_id,
        };

        // For now, we'll use the existing load_with_counts method and filter for removed comments
        let comments = DbComment::load_with_counts(
            pool,
            user_id_join,
            sort.into(),
            crate::ListingType::All.into(),
            Some(page as i64),
            Some(limit as i64),
            user_id,
            post_id,
            board_id,
            false, // saved_only
            None, // search_term
            false, // include_deleted
            include_removed || removed_only, // include_removed
            false, // include_banned_boards
            None, // parent_ids
            None, // max_depth
        )
        .await?;

        // Filter for removed comments if requested
        let filtered_comments = if removed_only {
            comments.into_iter()
                .filter(|(comment, _)| comment.is_removed)
                .collect()
        } else {
            comments
        };

        Ok(filtered_comments.into_iter().map(Comment::from).collect::<Vec<Comment>>())
    }
}