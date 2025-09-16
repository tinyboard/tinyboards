use tinyboards_db::{
    models::{
        board::{board_mods::BoardModerator, boards::Board},
        user::user::{AdminPerms, User},
    },
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

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
    user_id: i32,
    board_id: i32,
) -> Result<(), TinyBoardsError> {
    // First check if user is admin
    let user = User::read(pool, user_id).await?;
    if user.is_admin {
        return Ok(());
    }

    // Check if user is moderator of the board
    let board = Board::read(pool, board_id).await?;
    let is_mod = Board::board_get_mod(pool, board.id, user_id).await.is_ok();

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
    board_id: Option<i32>,
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