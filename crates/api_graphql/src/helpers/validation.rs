use crate::DbPool;
use tinyboards_db::models::board::board_mods::ModPerms;
use tinyboards_db::models::board::boards::Board as DbBoard;
use tinyboards_db::models::person::local_user::AdminPerms;
use tinyboards_db::models::site::local_site::LocalSite as DbLocalSite;
use tinyboards_db_views::structs::LocalUserView;
use tinyboards_utils::TinyBoardsError;

/// Check if the instance is private. Return an error if the instance is private and the user is not logged in.
pub async fn check_private_instance(
    user: Option<&LocalUserView>,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    if user.is_none() {
        let site = DbLocalSite::read(pool).await?;

        if site.private_instance {
            return Err(TinyBoardsError::from_message(
                403,
                &format!("{} is a private instance. You need an account.", &site.name),
            ));
        }
    }

    Ok(())
}

/**
 * Function to check if the user is a site admin or a moderator of a given board.
 *
 * Parameters:
 * 	- pool: DB connection pool
 * 	- v: a local user view
 * 	- board_id: ID of the board to check mod perms for
 * 	- with_permission: check for this mod permission
 * 	- or_admin_perms: optional, admin perms that bypass the mod check, fx. ModPerms::Content can be bypassed by AdminPerms::Content
 **/
pub async fn require_mod_or_admin(
    v: &LocalUserView,
    pool: &DbPool,
    board_id: i32,
    with_permission: ModPerms,
    or_admin_perms: Option<AdminPerms>,
) -> Result<(), TinyBoardsError> {
    let or_admin_perms = or_admin_perms.unwrap_or(AdminPerms::Full);
    if v.local_user.has_permission(or_admin_perms) {
        // user is admin
        Ok(())
    } else {
        // user is not admin: check mod permissions instead
        let m = DbBoard::board_get_mod(pool, board_id, v.person.id)
            .await?
            .ok_or(TinyBoardsError::from_message(
                403,
                "You are not a moderator or admin.",
            ))?;

        if m.has_permission(with_permission) {
            Ok(())
        } else {
            Err(TinyBoardsError::from_message(
                403,
                "Insufficient moderator permissions.",
            ))
        }
    }
}
