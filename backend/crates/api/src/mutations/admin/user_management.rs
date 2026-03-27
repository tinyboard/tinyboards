use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbModerationAction,
    models::{
        aggregates::UserAggregates as DbUserAggregates,
        moderator::moderation_log::ModerationLogInsertForm,
        user::user::{AdminPerms, User as DbUser, UserUpdateForm},
    },
    schema::{moderation_log, user_aggregates, users},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{helpers::permissions, structs::user::User};

#[derive(Default)]
pub struct UserManagement;

#[Object]
impl UserManagement {
    /// Update a user's board creation approval status (admin only).
    /// Can both grant and revoke board creation trust.
    pub async fn update_user_board_creation_approval(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        approved: bool,
    ) -> Result<User> {
        let pool = ctx.data::<DbPool>()?;
        let admin = permissions::require_admin_permission(ctx, AdminPerms::Users)?;

        let target_uuid: Uuid = user_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid user ID".to_string()))?;

        // Prevent modifying your own approval status
        if admin.id == target_uuid {
            return Err(TinyBoardsError::from_message(
                400,
                "Cannot modify your own board creation approval status",
            )
            .into());
        }

        let conn = &mut get_conn(pool).await?;

        // Verify target user exists
        let _target: DbUser = users::table
            .find(target_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("User not found".to_string()))?;

        // Apply the update
        let form = UserUpdateForm {
            is_board_creation_approved: Some(approved),
            ..Default::default()
        };

        let updated_user: DbUser = diesel::update(users::table.find(target_uuid))
            .set(&form)
            .get_result(conn)
            .await
            .map_err(|e| {
                TinyBoardsError::Database(format!(
                    "Failed to update user board creation approval: {}",
                    e
                ))
            })?;

        // Fetch aggregates for the response (return zeroed struct if not found)
        let agg: Option<DbUserAggregates> = user_aggregates::table
            .filter(user_aggregates::user_id.eq(target_uuid))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(User::from_db(updated_user, agg))
    }

    /// Set a user's admin level (owner/full-admin only).
    /// Level 0 removes admin privileges. Levels 1-7 grant progressively higher permissions.
    /// Only admins with a higher level than the target can modify the target's level,
    /// and they cannot set a level equal to or higher than their own.
    pub async fn set_user_admin_level(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        admin_level: i32,
    ) -> Result<User> {
        let pool = ctx.data::<DbPool>()?;
        let admin = permissions::require_admin_permission(ctx, AdminPerms::Full)?;
        let conn = &mut get_conn(pool).await?;

        let target_uuid: Uuid = user_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid user ID".to_string()))?;

        if admin.id == target_uuid {
            return Err(
                TinyBoardsError::from_message(400, "Cannot modify your own admin level").into(),
            );
        }

        if admin_level < 0 || admin_level > 7 {
            return Err(
                TinyBoardsError::from_message(400, "Admin level must be between 0 and 7").into(),
            );
        }

        // Owners (level 7) can promote others up to their own level.
        // All other admins can only assign levels strictly below their own.
        if admin.admin_level >= 7 {
            if admin_level > 7 {
                return Err(TinyBoardsError::from_message(
                    403,
                    "Admin level must be between 0 and 7",
                )
                .into());
            }
        } else if admin_level >= admin.admin_level {
            return Err(TinyBoardsError::from_message(
                403,
                "Cannot grant an admin level equal to or higher than your own",
            )
            .into());
        }

        let target: DbUser = users::table
            .find(target_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("User not found".to_string()))?;

        // Owners can modify other owners; lower admins cannot modify peers or superiors
        if target.is_admin && target.admin_level >= admin.admin_level && admin.admin_level < 7 {
            return Err(TinyBoardsError::from_message(
                403,
                "Cannot modify an admin with equal or higher permissions",
            )
            .into());
        }

        let is_admin = admin_level > 0;
        let form = UserUpdateForm {
            is_admin: Some(is_admin),
            admin_level: Some(admin_level),
            ..Default::default()
        };

        let updated_user: DbUser = diesel::update(users::table.find(target_uuid))
            .set(&form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to update admin level: {}", e)))?;

        // Log the action
        let action = if admin_level > 0 {
            DbModerationAction::AddAdmin
        } else {
            DbModerationAction::RemoveAdmin
        };

        let metadata = serde_json::json!({
            "target_username": target.name,
            "old_level": target.admin_level,
            "new_level": admin_level,
        });

        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: admin.id,
                action_type: action,
                target_type: "user".to_string(),
                target_id: target_uuid,
                board_id: None,
                reason: None,
                metadata: Some(metadata),
                expires_at: None,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let agg: Option<DbUserAggregates> = user_aggregates::table
            .filter(user_aggregates::user_id.eq(target_uuid))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(User::from_db(updated_user, agg))
    }

    /// Purge (hard delete) a user account (owner only).
    /// This is an irreversible action that removes the user from the database.
    pub async fn delete_account(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let admin = permissions::require_admin_permission(ctx, AdminPerms::Owner)?;
        let conn = &mut get_conn(pool).await?;

        let target_uuid: Uuid = user_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid user ID".to_string()))?;

        if admin.id == target_uuid {
            return Err(
                TinyBoardsError::from_message(400, "Cannot delete your own account from admin panel").into(),
            );
        }

        let target: DbUser = users::table
            .find(target_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("User not found".to_string()))?;

        if target.is_admin && target.admin_level >= admin.admin_level && admin.admin_level < 7 {
            return Err(TinyBoardsError::from_message(
                403,
                "Cannot delete an admin with equal or higher permissions",
            )
            .into());
        }

        // Soft-delete the account
        let form = UserUpdateForm {
            deleted_at: Some(Some(chrono::Utc::now())),
            is_banned: Some(true),
            ..Default::default()
        };

        diesel::update(users::table.find(target_uuid))
            .set(&form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to delete account: {}", e)))?;

        // Log the action
        let metadata = serde_json::json!({
            "target_username": target.name,
            "deleted_by": admin.name,
        });

        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: admin.id,
                action_type: DbModerationAction::PurgeUser,
                target_type: "user".to_string(),
                target_id: target_uuid,
                board_id: None,
                reason: None,
                metadata: Some(metadata),
                expires_at: None,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(true)
    }
}
