use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    schema::{comments, posts},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

/// Check if a post is deleted or removed
#[tracing::instrument(skip_all)]
pub async fn check_post_deleted_or_removed(
    post_id: Uuid,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    let conn = &mut get_conn(pool).await?;

    let (is_deleted_at, is_removed): (Option<chrono::DateTime<chrono::Utc>>, bool) = posts::table
        .find(post_id)
        .select((posts::deleted_at.nullable(), posts::is_removed))
        .first(conn)
        .await
        .map_err(|_e| TinyBoardsError::from_message(404, "couldn't find post"))?;

    if is_deleted_at.is_some() || is_removed {
        Err(TinyBoardsError::from_message(404, "post deleted or removed"))
    } else {
        Ok(())
    }
}

/// Check if a post is deleted, removed, or locked
#[tracing::instrument(skip_all)]
pub async fn check_post_deleted_removed_or_locked(
    post_id: Uuid,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    let conn = &mut get_conn(pool).await?;

    let (is_deleted_at, is_removed, is_locked): (Option<chrono::DateTime<chrono::Utc>>, bool, bool) =
        posts::table
            .find(post_id)
            .select((posts::deleted_at.nullable(), posts::is_removed, posts::is_locked))
            .first(conn)
            .await
            .map_err(|_e| TinyBoardsError::from_message(404, "couldn't find post"))?;

    if is_locked {
        Err(TinyBoardsError::from_message(403, "post locked"))
    } else if is_deleted_at.is_some() || is_removed {
        Err(TinyBoardsError::from_message(404, "post deleted or removed"))
    } else {
        Ok(())
    }
}

/// Check if a comment is deleted or removed
#[tracing::instrument(skip_all)]
pub async fn check_comment_deleted_or_removed(
    comment_id: Uuid,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    let conn = &mut get_conn(pool).await?;

    let (is_deleted_at, is_removed): (Option<chrono::DateTime<chrono::Utc>>, bool) = comments::table
        .find(comment_id)
        .select((comments::deleted_at.nullable(), comments::is_removed))
        .first(conn)
        .await
        .map_err(|_e| TinyBoardsError::from_message(404, "couldn't find comment"))?;

    if is_deleted_at.is_some() || is_removed {
        Err(TinyBoardsError::from_message(404, "comment deleted or removed"))
    } else {
        Ok(())
    }
}
