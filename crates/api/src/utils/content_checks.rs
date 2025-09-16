use tinyboards_db::{
    models::{comment::comments::Comment, post::posts::Post},
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

/// Check if a post is deleted or removed
#[tracing::instrument(skip_all)]
pub async fn check_post_deleted_or_removed(
    post_id: i32,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    let post = Post::read(pool, post_id)
        .await
        .map_err(|_e| TinyBoardsError::from_message(404, "couldn't find post"))?;

    if post.is_deleted || post.is_removed {
        Err(TinyBoardsError::from_message(404, "post deleted or removed"))
    } else {
        Ok(())
    }
}

/// Check if a post is deleted, removed, or locked
#[tracing::instrument(skip_all)]
pub async fn check_post_deleted_removed_or_locked(
    post_id: i32,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    let post = Post::read(pool, post_id)
        .await
        .map_err(|_e| TinyBoardsError::from_message(404, "couldn't find post"))?;

    if post.is_locked {
        Err(TinyBoardsError::from_message(403, "post locked"))
    } else if post.is_deleted || post.is_removed {
        Err(TinyBoardsError::from_message(404, "post deleted or removed"))
    } else {
        Ok(())
    }
}

/// Check if a comment is deleted or removed
#[tracing::instrument(skip_all)]
pub async fn check_comment_deleted_or_removed(
    comment_id: i32,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    let comment = Comment::read(pool, comment_id)
        .await
        .map_err(|_e| TinyBoardsError::from_message(404, "couldn't find comment"))?;

    if comment.is_deleted || comment.is_removed {
        Err(TinyBoardsError::from_message(404, "comment deleted or removed"))
    } else {
        Ok(())
    }
}