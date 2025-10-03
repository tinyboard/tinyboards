use async_graphql::*;
use tinyboards_db::{
    models::user::{
        user::{User as DbUser, UserForm, AdminPerms},
    },
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

use crate::{LoggedInUser, structs::user::User};

#[derive(Default)]
pub struct UserManagement;

#[Object]
impl UserManagement {
    /// Update user's board creation approval status (admin only)
    /// This can both grant and revoke board creation trust
    pub async fn update_user_board_creation_approval(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
        approved: bool,
    ) -> Result<User> {
        let pool = ctx.data::<DbPool>()?;
        let admin = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        // Check if user has Users permission (can manage users)
        if !admin.has_permission(AdminPerms::Users) {
            return Err(TinyBoardsError::from_message(
                403,
                "You need Users permission to manage board creation approvals"
            ).into());
        }

        // Check if trying to modify yourself
        if admin.id == user_id {
            return Err(TinyBoardsError::from_message(
                400,
                "Cannot modify your own board creation approval status"
            ).into());
        }

        // Get the target user
        let target_user = DbUser::read(pool, user_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "User not found"))?;

        // Update the user's board creation approval
        let form = UserForm {
            board_creation_approved: Some(approved),
            ..Default::default()
        };

        let updated_user = DbUser::update(pool, user_id, &form)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(
                e,
                500,
                "Failed to update user board creation approval"
            ))?;

        // Fetch user aggregates for the response
        use tinyboards_db::aggregates::structs::UserAggregates;
        let user_aggregates = UserAggregates::read(pool, updated_user.id)
            .await
            .unwrap_or_else(|_| UserAggregates {
                id: 0,
                user_id: updated_user.id,
                post_count: 0,
                post_score: 0,
                comment_count: 0,
                comment_score: 0,
            });

        Ok(User::from((updated_user, user_aggregates)))
    }
}
