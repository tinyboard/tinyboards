use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::user::user::{AdminPerms, User},
    schema::{board_moderators, users},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

/// Check if a user is an admin
pub fn is_admin(user: &User) -> Result<(), TinyBoardsError> {
    if !user.is_admin {
        return Err(TinyBoardsError::from_message(403, "not an admin"));
    }
    Ok(())
}

/// Check if a user has specific admin permissions
pub fn check_admin_permission(user: &User, perm: AdminPerms) -> Result<(), TinyBoardsError> {
    if !user.has_permission(perm) {
        return Err(TinyBoardsError::from_message(403, "insufficient admin permissions"));
    }
    Ok(())
}

/// Check if a user is a moderator of a board or an admin
#[tracing::instrument(skip_all)]
pub async fn is_mod_or_admin(
    pool: &DbPool,
    user_id: Uuid,
    board_id: Uuid,
) -> Result<(), TinyBoardsError> {
    let conn = &mut get_conn(pool).await?;

    // Check if user is admin
    let user: User = users::table
        .find(user_id)
        .first(conn)
        .await
        .map_err(|_| TinyBoardsError::from_message(404, "user not found"))?;

    if user.is_admin {
        return Ok(());
    }

    // Check if user is moderator of the board
    let is_mod: bool = board_moderators::table
        .filter(board_moderators::board_id.eq(board_id))
        .filter(board_moderators::user_id.eq(user_id))
        .filter(board_moderators::is_invite_accepted.eq(true))
        .count()
        .get_result::<i64>(conn)
        .await
        .map(|c| c > 0)
        .unwrap_or(false);

    if !is_mod {
        return Err(TinyBoardsError::from_message(403, "not a mod or admin"));
    }
    Ok(())
}

/// Check if a user is a moderator of a board or an admin (optional user)
#[tracing::instrument(skip_all)]
pub async fn is_mod_or_admin_opt(
    pool: &DbPool,
    user: Option<&User>,
    board_id: Option<Uuid>,
) -> Result<(), TinyBoardsError> {
    if let Some(user) = user {
        if let Some(board_id) = board_id {
            is_mod_or_admin(pool, user.id, board_id).await
        } else {
            is_admin(user)
        }
    } else {
        Err(TinyBoardsError::from_message(403, "not a mod or admin"))
    }
}
