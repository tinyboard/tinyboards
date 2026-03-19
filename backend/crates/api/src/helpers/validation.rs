use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        user::user::{AdminPerms, User},
    },
    schema::{board_moderators, site},
    utils::get_conn,
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::DbPool;

/// Check if the instance is private. Returns an error if the instance is
/// private and the user is not logged in.
pub async fn check_private_instance(
    user: Option<&User>,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    if user.is_none() {
        let conn = &mut get_conn(pool).await?;
        let is_private: bool = site::table
            .select(site::is_private)
            .first(conn)
            .await
            .unwrap_or(false);

        if is_private {
            return Err(TinyBoardsError::Forbidden(
                "This is a private instance. You need an account.".to_string(),
            ));
        }
    }
    Ok(())
}

/// Check if the user is a site admin or a moderator of a given board.
///
/// Parameters:
///   - `v`: the authenticated user
///   - `pool`: DB connection pool
///   - `board_id`: ID of the board to check mod permissions for
///   - `with_permission`: the required mod permission
///   - `or_admin_perms`: admin permission that bypasses the mod check;
///     defaults to `AdminPerms::Full` if `None`
pub async fn require_mod_or_admin(
    v: &User,
    pool: &DbPool,
    board_id: Uuid,
    with_permission: ModPerms,
    or_admin_perms: Option<AdminPerms>,
) -> Result<(), TinyBoardsError> {
    let or_admin_perms = or_admin_perms.unwrap_or(AdminPerms::Full);

    if v.has_permission(or_admin_perms) {
        return Ok(());
    }

    // User is not an admin with sufficient perms -- check mod status.
    let conn = &mut get_conn(pool).await?;
    let mod_row: Option<BoardModerator> = board_moderators::table
        .filter(board_moderators::board_id.eq(board_id))
        .filter(board_moderators::user_id.eq(v.id))
        .first::<BoardModerator>(conn)
        .await
        .optional()
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    match mod_row {
        Some(m) => {
            if m.has_permission(with_permission) {
                Ok(())
            } else {
                Err(TinyBoardsError::Forbidden(
                    "Insufficient moderator permissions".to_string(),
                ))
            }
        }
        None => Err(TinyBoardsError::Forbidden(
            "You are not a moderator or admin".to_string(),
        )),
    }
}
