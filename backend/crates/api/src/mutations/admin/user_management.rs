use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        aggregates::UserAggregates as DbUserAggregates,
        user::user::{AdminPerms, User as DbUser, UserUpdateForm},
    },
    schema::{user_aggregates, users},
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
}
