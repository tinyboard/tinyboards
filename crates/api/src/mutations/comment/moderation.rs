use crate::helpers::validation::require_mod_or_admin;
use crate::structs::comment::Comment;
use crate::DbPool;
use crate::LoggedInUser;
use async_graphql::*;
use tinyboards_db::models::board::board_mods::ModPerms;
use tinyboards_db::models::comment::comments::Comment as DbComment;
use tinyboards_db::models::user::user::AdminPerms;
use tinyboards_db::traits::Crud;
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct CommentModeration;

#[Object]
impl CommentModeration {
    pub async fn set_comment_removed(
        &self,
        ctx: &Context<'_>,
        id: i32,
        value: bool,
    ) -> Result<Comment> {
        let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        let comment = DbComment::read(pool, id).await?;

        require_mod_or_admin(
            v,
            pool,
            comment.board_id,
            ModPerms::Content,
            Some(AdminPerms::Content),
        )
        .await?;

        DbComment::update_removed(pool, comment.id, value).await?;
        // mark reports as resolved
        DbComment::resolve_reports(pool, comment.id, v.id).await?;
        let res = DbComment::get_with_counts(pool, comment.id).await?;

        Ok(Comment::from(res))
    }

    pub async fn set_comment_pinned(
        &self,
        ctx: &Context<'_>,
        id: i32,
        value: bool,
    ) -> Result<Comment> {
        let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        let comment = DbComment::read(pool, id).await?;

        require_mod_or_admin(
            v,
            pool,
            comment.board_id,
            ModPerms::Content,
            Some(AdminPerms::Content),
        )
        .await?;

        if comment.is_deleted || comment.is_removed {
            return Err(TinyBoardsError::from_message(404, "Comment deleted or removed.").into());
        }

        // Only top-level comments can be pinned
        if comment.level > 1 {
            return Err(
                TinyBoardsError::from_message(400, "You can only pin top-level comments.").into(),
            );
        }

        DbComment::update_pinned(pool, comment.id, comment.post_id, value).await?;
        let res = DbComment::get_with_counts(pool, comment.id).await?;

        Ok(Comment::from(res))
    }
}
