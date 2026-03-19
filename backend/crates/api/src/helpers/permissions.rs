use async_graphql::Context;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        user::user::{AdminPerms, User},
    },
    schema::board_moderators,
    utils::get_conn,
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{DbPool, LoggedInUser};

/// Get the logged-in user from GraphQL context. Returns error if not authenticated.
pub fn require_auth<'a>(ctx: &'a Context<'_>) -> Result<&'a User, TinyBoardsError> {
    ctx.data::<LoggedInUser>()
        .map_err(|_| TinyBoardsError::Unauthorized)?
        .inner()
        .ok_or(TinyBoardsError::Unauthorized)
}

/// Get the logged-in user, also checking they are not banned or deleted.
pub fn require_auth_not_banned<'a>(ctx: &'a Context<'_>) -> Result<&'a User, TinyBoardsError> {
    let user = require_auth(ctx)?;
    if user.is_banned {
        return Err(TinyBoardsError::Forbidden(
            "Your account is banned".to_string(),
        ));
    }
    if user.deleted_at.is_some() {
        return Err(TinyBoardsError::Forbidden(
            "Account is deleted".to_string(),
        ));
    }
    Ok(user)
}

/// Require the user to be a site admin at any level.
pub fn require_admin<'a>(ctx: &'a Context<'_>) -> Result<&'a User, TinyBoardsError> {
    let user = require_auth_not_banned(ctx)?;
    if !user.is_admin {
        return Err(TinyBoardsError::Forbidden(
            "Admin access required".to_string(),
        ));
    }
    Ok(user)
}

/// Require the user to have a specific admin permission level.
pub fn require_admin_permission<'a>(
    ctx: &'a Context<'_>,
    perm: AdminPerms,
) -> Result<&'a User, TinyBoardsError> {
    let user = require_auth_not_banned(ctx)?;
    if !user.has_permission(perm) {
        return Err(TinyBoardsError::Forbidden(
            "Insufficient admin permissions".to_string(),
        ));
    }
    Ok(user)
}

/// Require the user to be a moderator of a specific board (with a specific
/// permission), OR a site admin with the fallback admin permission.
///
/// If `admin_fallback` is `None`, defaults to `AdminPerms::Full`.
pub async fn require_board_mod_or_admin<'a>(
    ctx: &'a Context<'_>,
    pool: &DbPool,
    board_id: Uuid,
    mod_perm: ModPerms,
    admin_fallback: Option<AdminPerms>,
) -> Result<&'a User, TinyBoardsError> {
    let user = require_auth_not_banned(ctx)?;
    let admin_fallback = admin_fallback.unwrap_or(AdminPerms::Full);

    // Admin with sufficient permissions bypasses mod check.
    if user.has_permission(admin_fallback) {
        return Ok(user);
    }

    // Not an admin with sufficient perms -- check board moderator status.
    let conn = &mut get_conn(pool).await?;
    let mod_row: Option<BoardModerator> = board_moderators::table
        .filter(board_moderators::board_id.eq(board_id))
        .filter(board_moderators::user_id.eq(user.id))
        .first::<BoardModerator>(conn)
        .await
        .optional()
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    match mod_row {
        Some(m) => {
            if m.has_permission(mod_perm) {
                Ok(user)
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

/// Get the optional logged-in user (returns None if not authenticated).
pub fn optional_auth<'a>(ctx: &'a Context<'_>) -> Option<&'a User> {
    ctx.data::<LoggedInUser>().ok().and_then(|l| l.inner())
}
