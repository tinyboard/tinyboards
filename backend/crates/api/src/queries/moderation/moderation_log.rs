use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbModerationAction,
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        moderator::moderation_log::ModerationLog as DbModerationLog,
        user::user::AdminPerms,
    },
    schema::{board_moderators, moderation_log, users},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::LoggedInUser;

#[derive(Default)]
pub struct ModerationLogQueries;

#[derive(SimpleObject)]
pub struct ModerationLogEntry {
    pub id: ID,
    pub moderator_id: ID,
    pub moderator_name: String,
    pub action_type: String,
    pub target_type: String,
    pub target_id: ID,
    pub board_id: Option<ID>,
    pub reason: Option<String>,
    pub metadata: Option<String>,
    #[graphql(name = "createdAt")]
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(SimpleObject)]
pub struct ModerationLogResponse {
    pub entries: Vec<ModerationLogEntry>,
    pub total_count: i32,
}

fn action_type_str(action: &DbModerationAction) -> &'static str {
    match action {
        DbModerationAction::BanUser => "ban_user",
        DbModerationAction::UnbanUser => "unban_user",
        DbModerationAction::BanFromBoard => "ban_from_board",
        DbModerationAction::UnbanFromBoard => "unban_from_board",
        DbModerationAction::RemovePost => "remove_post",
        DbModerationAction::RestorePost => "restore_post",
        DbModerationAction::RemoveComment => "remove_comment",
        DbModerationAction::RestoreComment => "restore_comment",
        DbModerationAction::LockPost => "lock_post",
        DbModerationAction::UnlockPost => "unlock_post",
        DbModerationAction::LockComment => "lock_comment",
        DbModerationAction::UnlockComment => "unlock_comment",
        DbModerationAction::FeaturePost => "feature_post",
        DbModerationAction::UnfeaturePost => "unfeature_post",
        DbModerationAction::RemoveBoard => "remove_board",
        DbModerationAction::RestoreBoard => "restore_board",
        DbModerationAction::HideBoard => "hide_board",
        DbModerationAction::UnhideBoard => "unhide_board",
        DbModerationAction::AddMod => "add_mod",
        DbModerationAction::RemoveMod => "remove_mod",
        DbModerationAction::AddAdmin => "add_admin",
        DbModerationAction::RemoveAdmin => "remove_admin",
        DbModerationAction::PurgeUser => "purge_user",
        DbModerationAction::PurgePost => "purge_post",
        DbModerationAction::PurgeComment => "purge_comment",
        DbModerationAction::PurgeBoard => "purge_board",
    }
}

#[Object]
impl ModerationLogQueries {
    /// Get moderation log with optional filters
    pub async fn get_moderation_log(
        &self,
        ctx: &Context<'_>,
        board_id: Option<ID>,
        action_type: Option<String>,
        moderator_id: Option<ID>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<ModerationLogResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let is_admin = user.has_permission(AdminPerms::Content);

        let board_uuid: Option<Uuid> = if let Some(ref bid) = board_id {
            Some(bid.parse().map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?)
        } else {
            None
        };

        if !is_admin && board_uuid.is_none() {
            return Err(TinyBoardsError::from_message(403, "You must specify a board_id if you're not an admin").into());
        }

        if let Some(bid) = board_uuid {
            if !is_admin {
                let moderator: BoardModerator = board_moderators::table
                    .filter(board_moderators::board_id.eq(bid))
                    .filter(board_moderators::user_id.eq(user.id))
                    .filter(board_moderators::is_invite_accepted.eq(true))
                    .first(conn)
                    .await
                    .map_err(|_| TinyBoardsError::from_message(403, "You are not a moderator of this board"))?;

                if !moderator.has_permission(ModPerms::Content) {
                    return Err(TinyBoardsError::from_message(403, "Insufficient moderation permissions").into());
                }
            }
        }

        // Build query
        let mut query = moderation_log::table
            .order(moderation_log::created_at.desc())
            .into_boxed();

        if let Some(bid) = board_uuid {
            query = query.filter(moderation_log::board_id.eq(bid));
        }

        if let Some(ref mid) = moderator_id {
            let mod_uuid: Uuid = mid.parse().map_err(|_| TinyBoardsError::NotFound("Invalid moderator ID".into()))?;
            query = query.filter(moderation_log::moderator_id.eq(mod_uuid));
        }

        // Filter by action_type string matching the enum
        if let Some(ref at) = action_type {
            if let Some(db_action) = parse_action_type(at) {
                query = query.filter(moderation_log::action_type.eq(db_action));
            }
        }

        let logs: Vec<DbModerationLog> = query
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Get moderator names via a single query
        let mod_ids: Vec<Uuid> = logs.iter().map(|l| l.moderator_id).collect();
        let mod_names: Vec<(Uuid, String)> = users::table
            .filter(users::id.eq_any(&mod_ids))
            .select((users::id, users::name))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let mod_name_map: std::collections::HashMap<Uuid, String> = mod_names.into_iter().collect();

        let entries: Vec<ModerationLogEntry> = logs.iter().map(|log| {
            let moderator_name = mod_name_map
                .get(&log.moderator_id)
                .cloned()
                .unwrap_or_else(|| "Deleted User".to_string());

            ModerationLogEntry {
                id: log.id.to_string().into(),
                moderator_id: log.moderator_id.to_string().into(),
                moderator_name,
                action_type: action_type_str(&log.action_type).to_string(),
                target_type: log.target_type.clone(),
                target_id: log.target_id.to_string().into(),
                board_id: log.board_id.map(|id| id.to_string().into()),
                reason: log.reason.clone(),
                metadata: log.metadata.as_ref().map(|v| v.to_string()),
                created_at: log.created_at.to_string(),
                expires_at: log.expires_at.map(|dt| dt.to_string()),
            }
        }).collect();

        let total_count = entries.len() as i32;

        Ok(ModerationLogResponse {
            entries,
            total_count,
        })
    }

    /// Get moderation actions for a specific target (admin only)
    pub async fn get_target_moderation_history(
        &self,
        ctx: &Context<'_>,
        target_type: String,
        target_id: ID,
        limit: Option<i64>,
    ) -> Result<Vec<ModerationLogEntry>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let is_admin = user.has_permission(AdminPerms::Content);
        if !is_admin {
            return Err(TinyBoardsError::from_message(403, "Admin privileges required to view target moderation history").into());
        }

        let target_uuid: Uuid = target_id.parse().map_err(|_| TinyBoardsError::NotFound("Invalid target ID".into()))?;
        let limit = limit.unwrap_or(50).min(100);

        let logs: Vec<DbModerationLog> = moderation_log::table
            .filter(moderation_log::target_type.eq(&target_type))
            .filter(moderation_log::target_id.eq(target_uuid))
            .order(moderation_log::created_at.desc())
            .limit(limit)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let mod_ids: Vec<Uuid> = logs.iter().map(|l| l.moderator_id).collect();
        let mod_names: Vec<(Uuid, String)> = users::table
            .filter(users::id.eq_any(&mod_ids))
            .select((users::id, users::name))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let mod_name_map: std::collections::HashMap<Uuid, String> = mod_names.into_iter().collect();

        Ok(logs.iter().map(|log| {
            let moderator_name = mod_name_map
                .get(&log.moderator_id)
                .cloned()
                .unwrap_or_else(|| "Deleted User".to_string());

            ModerationLogEntry {
                id: log.id.to_string().into(),
                moderator_id: log.moderator_id.to_string().into(),
                moderator_name,
                action_type: action_type_str(&log.action_type).to_string(),
                target_type: log.target_type.clone(),
                target_id: log.target_id.to_string().into(),
                board_id: log.board_id.map(|id| id.to_string().into()),
                reason: log.reason.clone(),
                metadata: log.metadata.as_ref().map(|v| v.to_string()),
                created_at: log.created_at.to_string(),
                expires_at: log.expires_at.map(|dt| dt.to_string()),
            }
        }).collect())
    }
}

fn parse_action_type(s: &str) -> Option<DbModerationAction> {
    match s {
        "ban_user" => Some(DbModerationAction::BanUser),
        "unban_user" => Some(DbModerationAction::UnbanUser),
        "ban_from_board" => Some(DbModerationAction::BanFromBoard),
        "unban_from_board" => Some(DbModerationAction::UnbanFromBoard),
        "remove_post" => Some(DbModerationAction::RemovePost),
        "restore_post" => Some(DbModerationAction::RestorePost),
        "remove_comment" => Some(DbModerationAction::RemoveComment),
        "restore_comment" => Some(DbModerationAction::RestoreComment),
        "lock_post" => Some(DbModerationAction::LockPost),
        "unlock_post" => Some(DbModerationAction::UnlockPost),
        "lock_comment" => Some(DbModerationAction::LockComment),
        "unlock_comment" => Some(DbModerationAction::UnlockComment),
        "feature_post" => Some(DbModerationAction::FeaturePost),
        "unfeature_post" => Some(DbModerationAction::UnfeaturePost),
        "remove_board" => Some(DbModerationAction::RemoveBoard),
        _ => None,
    }
}
