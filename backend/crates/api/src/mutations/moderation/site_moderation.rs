use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbModerationAction,
    models::{
        moderator::moderation_log::ModerationLogInsertForm,
        social::{UserBan, UserBanInsertForm},
        user::user::{AdminPerms, User as DbUser, UserUpdateForm},
    },
    schema::{moderation_log, user_bans, users},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::LoggedInUser;

#[derive(Default)]
pub struct SiteModerationMutations;

#[derive(SimpleObject)]
pub struct BanUserResponse {
    pub success: bool,
    pub ban_id: ID,
    pub message: String,
}

#[derive(SimpleObject)]
pub struct UnbanUserResponse {
    pub success: bool,
    pub message: String,
}

#[derive(InputObject)]
pub struct BanUserInput {
    pub user_id: ID,
    pub reason: Option<String>,
    pub expires_days: Option<i32>,
}

#[Object]
impl SiteModerationMutations {
    /// Ban a user site-wide (admin only)
    pub async fn ban_user_from_site(
        &self,
        ctx: &Context<'_>,
        input: BanUserInput,
    ) -> Result<BanUserResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        if !user.has_permission(AdminPerms::Content) {
            return Err(TinyBoardsError::from_message(403, "Admin privileges required to ban users site-wide").into());
        }

        let target_id: Uuid = input.user_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid user ID".into()))?;

        if user.id == target_id {
            return Err(TinyBoardsError::from_message(400, "You cannot ban yourself").into());
        }

        // Check if target user exists
        let target_user: DbUser = users::table
            .find(target_id)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("User not found".into()))?;

        if target_user.has_permission(AdminPerms::Content) {
            return Err(TinyBoardsError::from_message(403, "Cannot ban other administrators").into());
        }

        // Check if already banned
        let already_banned: bool = user_bans::table
            .filter(user_bans::user_id.eq(target_id))
            .filter(
                user_bans::expires_at.is_null()
                    .or(user_bans::expires_at.gt(chrono::Utc::now()))
            )
            .count()
            .get_result::<i64>(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?
            > 0;

        if already_banned {
            return Err(TinyBoardsError::from_message(400, "User is already banned").into());
        }

        let now = chrono::Utc::now();
        let expires_at = input.expires_days.map(|days| now + chrono::Duration::days(days as i64));

        // Create the ban record
        let ban_form = UserBanInsertForm {
            user_id: target_id,
            banned_by: Some(user.id),
            reason: input.reason.clone(),
            expires_at,
            banned_at: now,
        };

        let ban: UserBan = diesel::insert_into(user_bans::table)
            .values(&ban_form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Update user's banned status
        diesel::update(users::table.find(target_id))
            .set(&UserUpdateForm {
                is_banned: Some(true),
                ..Default::default()
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Log the moderation action
        let metadata = serde_json::json!({
            "target_username": target_user.name,
            "expires_at": expires_at,
            "is_permanent": expires_at.is_none(),
        });

        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: user.id,
                action_type: DbModerationAction::BanUser,
                target_type: "user".to_string(),
                target_id,
                board_id: None,
                reason: input.reason,
                metadata: Some(metadata),
                expires_at,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let message = if expires_at.is_some() {
            format!("User {} has been temporarily banned", target_user.name)
        } else {
            format!("User {} has been permanently banned", target_user.name)
        };

        Ok(BanUserResponse {
            success: true,
            ban_id: ban.id.to_string().into(),
            message,
        })
    }

    /// Unban a user site-wide (admin only)
    pub async fn unban_user_from_site(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        reason: Option<String>,
    ) -> Result<UnbanUserResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        if !user.has_permission(AdminPerms::Content) {
            return Err(TinyBoardsError::from_message(403, "Admin privileges required to unban users").into());
        }

        let target_id: Uuid = user_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid user ID".into()))?;

        let target_user: DbUser = users::table
            .find(target_id)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("User not found".into()))?;

        // Remove active bans
        let rows_affected = diesel::delete(
            user_bans::table
                .filter(user_bans::user_id.eq(target_id))
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if rows_affected == 0 {
            return Err(TinyBoardsError::from_message(400, "User is not currently banned").into());
        }

        // Update user's banned status
        diesel::update(users::table.find(target_id))
            .set(&UserUpdateForm {
                is_banned: Some(false),
                ..Default::default()
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Log the moderation action
        let metadata = serde_json::json!({
            "target_username": target_user.name,
            "unbanned_by": user.name,
        });

        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: user.id,
                action_type: DbModerationAction::UnbanUser,
                target_type: "user".to_string(),
                target_id,
                board_id: None,
                reason,
                metadata: Some(metadata),
                expires_at: None,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(UnbanUserResponse {
            success: true,
            message: format!("User {} has been unbanned", target_user.name),
        })
    }
}
