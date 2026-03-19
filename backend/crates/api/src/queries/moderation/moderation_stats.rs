use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use tinyboards_db::{
    enums::{DbModerationAction, DbReportStatus},
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        moderator::moderation_log::ModerationLog as DbModerationLog,
        user::user::AdminPerms,
    },
    schema::{board_moderators, comment_reports, moderation_log, post_reports, user_bans, users},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::LoggedInUser;

#[derive(Default)]
pub struct ModerationStatsQueries;

#[derive(SimpleObject)]
pub struct ModerationStatsResponse {
    pub total_actions: i32,
    pub actions_today: i32,
    pub actions_this_week: i32,
    pub pending_reports: i32,
    pub banned_users: i32,
    pub top_moderators: Vec<ModeratorActivity>,
    pub action_breakdown: Vec<ActionTypeCount>,
}

#[derive(SimpleObject)]
pub struct ModeratorActivity {
    pub moderator_id: ID,
    pub moderator_name: String,
    pub action_count: i32,
    pub last_action: Option<String>,
}

#[derive(SimpleObject)]
pub struct ActionTypeCount {
    pub action_type: String,
    pub count: i32,
    pub percentage: f64,
}

#[derive(SimpleObject)]
pub struct ModerationOverview {
    pub total_actions: i32,
    pub actions_today: i32,
    pub actions_this_week: i32,
    pub pending_reports: i32,
    pub banned_users: i32,
}

fn action_to_str(action: &DbModerationAction) -> &'static str {
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
impl ModerationStatsQueries {
    /// Get moderation statistics (BUG-032 fix: proper date handling, no unwrap)
    pub async fn get_moderation_stats(
        &self,
        ctx: &Context<'_>,
        board_id: Option<ID>,
        days: Option<i32>,
    ) -> Result<ModerationStatsResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let days = days.unwrap_or(30).min(365);

        let board_uuid: Option<Uuid> = if let Some(ref bid) = board_id {
            Some(bid.parse().map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?)
        } else {
            None
        };

        // Permission check
        let is_admin = user.has_permission(AdminPerms::Content);
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

        // BUG-043 fix: use chrono::Duration safely
        let now = chrono::Utc::now();
        let since_date = now - chrono::Duration::days(days as i64);
        let today_start = now.date_naive().and_hms_opt(0, 0, 0)
            .map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc))
            .unwrap_or(now);
        let week_start = now - chrono::Duration::days(7);

        // Get recent logs
        let mut query = moderation_log::table
            .filter(moderation_log::created_at.ge(since_date))
            .order(moderation_log::created_at.desc())
            .into_boxed();

        if let Some(bid) = board_uuid {
            query = query.filter(moderation_log::board_id.eq(bid));
        }

        let recent_logs: Vec<DbModerationLog> = query
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let total_actions = recent_logs.len() as i32;

        let actions_today = recent_logs.iter()
            .filter(|log| log.created_at >= today_start)
            .count() as i32;

        let actions_this_week = recent_logs.iter()
            .filter(|log| log.created_at >= week_start)
            .count() as i32;

        // Get pending reports count
        let post_report_query = post_reports::table
            .filter(post_reports::status.eq(DbReportStatus::Pending))
            .into_boxed();

        // For board-specific stats, we'd need to join but keep it simple
        let pending_post_reports: i64 = post_report_query
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let pending_comment_reports: i64 = comment_reports::table
            .filter(comment_reports::status.eq(DbReportStatus::Pending))
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let pending_reports = (pending_post_reports + pending_comment_reports) as i32;

        // Get banned users count
        let banned_users: i64 = user_bans::table
            .filter(
                user_bans::expires_at.is_null()
                    .or(user_bans::expires_at.gt(now))
            )
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Calculate moderator activity (BUG-032 fix: no unwrap on date comparison)
        let mut moderator_activity: HashMap<Uuid, (i32, Option<chrono::DateTime<chrono::Utc>>)> = HashMap::new();

        for log in &recent_logs {
            let entry = moderator_activity.entry(log.moderator_id).or_insert((0, None));
            entry.0 += 1;
            match entry.1 {
                None => entry.1 = Some(log.created_at),
                Some(prev) if log.created_at > prev => entry.1 = Some(log.created_at),
                _ => {}
            }
        }

        // Get moderator names
        let mod_ids: Vec<Uuid> = moderator_activity.keys().cloned().collect();
        let mod_names: Vec<(Uuid, String)> = if !mod_ids.is_empty() {
            users::table
                .filter(users::id.eq_any(&mod_ids))
                .select((users::id, users::name))
                .load(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?
        } else {
            vec![]
        };

        let mod_name_map: HashMap<Uuid, String> = mod_names.into_iter().collect();

        let mut top_moderators: Vec<ModeratorActivity> = moderator_activity
            .into_iter()
            .map(|(mod_id, (count, last))| {
                ModeratorActivity {
                    moderator_id: mod_id.to_string().into(),
                    moderator_name: mod_name_map.get(&mod_id).cloned().unwrap_or_else(|| "Deleted User".to_string()),
                    action_count: count,
                    last_action: last.map(|dt| dt.to_string()),
                }
            })
            .collect();

        top_moderators.sort_by(|a, b| b.action_count.cmp(&a.action_count));
        top_moderators.truncate(10);

        // Calculate action type breakdown
        let mut action_counts: HashMap<String, i32> = HashMap::new();
        for log in &recent_logs {
            *action_counts.entry(action_to_str(&log.action_type).to_string()).or_insert(0) += 1;
        }

        let mut action_breakdown: Vec<ActionTypeCount> = action_counts
            .into_iter()
            .map(|(action_type, count)| {
                let percentage = if total_actions > 0 {
                    (count as f64 / total_actions as f64) * 100.0
                } else {
                    0.0
                };
                ActionTypeCount { action_type, count, percentage }
            })
            .collect();

        action_breakdown.sort_by(|a, b| b.count.cmp(&a.count));

        Ok(ModerationStatsResponse {
            total_actions,
            actions_today,
            actions_this_week,
            pending_reports,
            banned_users: banned_users as i32,
            top_moderators,
            action_breakdown,
        })
    }

    /// Get quick moderation overview
    pub async fn get_moderation_overview(
        &self,
        ctx: &Context<'_>,
        board_id: Option<ID>,
    ) -> Result<ModerationOverview> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let board_uuid: Option<Uuid> = if let Some(ref bid) = board_id {
            Some(bid.parse().map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?)
        } else {
            None
        };

        let is_admin = user.has_permission(AdminPerms::Content);
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

        let now = chrono::Utc::now();
        let today_start = now.date_naive().and_hms_opt(0, 0, 0)
            .map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc))
            .unwrap_or(now);
        let week_start = now - chrono::Duration::days(7);

        // Total actions
        let mut query = moderation_log::table.into_boxed();
        if let Some(bid) = board_uuid {
            query = query.filter(moderation_log::board_id.eq(bid));
        }
        let total_actions: i64 = query.count().get_result(conn).await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Actions today
        let mut query = moderation_log::table
            .filter(moderation_log::created_at.ge(today_start))
            .into_boxed();
        if let Some(bid) = board_uuid {
            query = query.filter(moderation_log::board_id.eq(bid));
        }
        let actions_today: i64 = query.count().get_result(conn).await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Actions this week
        let mut query = moderation_log::table
            .filter(moderation_log::created_at.ge(week_start))
            .into_boxed();
        if let Some(bid) = board_uuid {
            query = query.filter(moderation_log::board_id.eq(bid));
        }
        let actions_this_week: i64 = query.count().get_result(conn).await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Pending reports
        let pending_post_reports: i64 = post_reports::table
            .filter(post_reports::status.eq(DbReportStatus::Pending))
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let pending_comment_reports: i64 = comment_reports::table
            .filter(comment_reports::status.eq(DbReportStatus::Pending))
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Banned users
        let banned_users: i64 = user_bans::table
            .filter(
                user_bans::expires_at.is_null()
                    .or(user_bans::expires_at.gt(now))
            )
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(ModerationOverview {
            total_actions: total_actions as i32,
            actions_today: actions_today as i32,
            actions_this_week: actions_this_week as i32,
            pending_reports: (pending_post_reports + pending_comment_reports) as i32,
            banned_users: banned_users as i32,
        })
    }
}
