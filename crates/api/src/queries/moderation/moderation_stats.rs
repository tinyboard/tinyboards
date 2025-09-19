use async_graphql::*;
use tinyboards_db::{
    models::{
        moderator::moderation_log::{ModerationLog, ModerationStats as DbModerationStats},
        board::board_mods::{BoardModerator, ModPerms},
        user::user::{AdminPerms, User},
    },
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

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
    pub moderator_id: i32,
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
pub struct ModerationStats {
    pub total_actions: i32,
    pub actions_today: i32,
    pub actions_this_week: i32,
    pub pending_reports: i32,
    pub banned_users: i32,
}

impl From<DbModerationStats> for ModerationStats {
    fn from(db_stats: DbModerationStats) -> Self {
        Self {
            total_actions: db_stats.total_actions as i32,
            actions_today: db_stats.actions_today as i32,
            actions_this_week: db_stats.actions_this_week as i32,
            pending_reports: db_stats.pending_reports as i32,
            banned_users: db_stats.banned_users as i32,
        }
    }
}

#[Object]
impl ModerationStatsQueries {
    /// Get moderation statistics (admin/moderator only)
    pub async fn get_moderation_stats(
        &self,
        ctx: &Context<'_>,
        board_id: Option<i32>,
        days: Option<i32>, // Time period for statistics
    ) -> Result<ModerationStatsResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        let days = days.unwrap_or(30).min(365); // Max 1 year

        // Check if user is admin or moderator
        let is_admin = user.has_permission(AdminPerms::Content);

        if !is_admin && board_id.is_none() {
            return Err(TinyBoardsError::from_message(
                403,
                "You must specify a board_id if you're not an admin",
            )
            .into());
        }

        // If board_id is specified, check if user is a moderator of that board
        if let Some(board_id) = board_id {
            if !is_admin {
                match BoardModerator::get_by_user_id_for_board(pool, user.id, board_id, true).await {
                    Ok(moderator) => {
                        if !moderator.has_permission(ModPerms::Content) {
                            return Err(TinyBoardsError::from_message(
                                403,
                                "You don't have permission to view moderation stats for this board",
                            )
                            .into());
                        }
                    }
                    Err(_) => {
                        return Err(TinyBoardsError::from_message(
                            403,
                            "You are not a moderator of this board",
                        )
                        .into());
                    }
                }
            }
        }

        // Get basic stats from the moderation log
        let basic_stats = ModerationLog::get_stats(pool, board_id).await?;

        // Get all moderation actions for the specified period
        let since_date = chrono::Utc::now().naive_utc() - chrono::Duration::days(days as i64);
        let all_logs = ModerationLog::list(
            pool,
            board_id,
            None,
            None,
            None, // No limit
            None,
        ).await?;

        // Filter by date range
        let recent_logs: Vec<_> = all_logs.into_iter()
            .filter(|log| log.created_at >= since_date)
            .collect();

        // Calculate moderator activity
        use std::collections::HashMap;
        let mut moderator_activity: HashMap<i32, (i32, Option<chrono::NaiveDateTime>)> = HashMap::new();

        for log in &recent_logs {
            let entry = moderator_activity.entry(log.moderator_id).or_insert((0, None));
            entry.0 += 1;
            if entry.1.is_none() || log.created_at > entry.1.unwrap() {
                entry.1 = Some(log.created_at);
            }
        }

        // Get moderator names and create activity list
        let mut top_moderators = Vec::new();
        for (moderator_id, (count, last_action)) in moderator_activity {
            use tinyboards_db::models::user::user::User;
            let moderator_name = match User::read(pool, moderator_id).await {
                Ok(user) => user.name,
                Err(_) => "Deleted User".to_string(),
            };

            top_moderators.push(ModeratorActivity {
                moderator_id,
                moderator_name,
                action_count: count,
                last_action: last_action.map(|dt| dt.to_string()),
            });
        }

        // Sort by action count (descending)
        top_moderators.sort_by(|a, b| b.action_count.cmp(&a.action_count));
        top_moderators.truncate(10); // Top 10 moderators

        // Calculate action type breakdown
        let mut action_counts: HashMap<String, i32> = HashMap::new();
        let total_recent_actions = recent_logs.len() as i32;

        for log in &recent_logs {
            *action_counts.entry(log.action_type.clone()).or_insert(0) += 1;
        }

        let mut action_breakdown = Vec::new();
        for (action_type, count) in action_counts {
            let percentage = if total_recent_actions > 0 {
                (count as f64 / total_recent_actions as f64) * 100.0
            } else {
                0.0
            };

            action_breakdown.push(ActionTypeCount {
                action_type,
                count,
                percentage,
            });
        }

        // Sort by count (descending)
        action_breakdown.sort_by(|a, b| b.count.cmp(&a.count));

        Ok(ModerationStatsResponse {
            total_actions: basic_stats.total_actions as i32,
            actions_today: basic_stats.actions_today as i32,
            actions_this_week: basic_stats.actions_this_week as i32,
            pending_reports: basic_stats.pending_reports as i32,
            banned_users: basic_stats.banned_users as i32,
            top_moderators,
            action_breakdown,
        })
    }

    /// Get quick moderation overview (admin/moderator only)
    pub async fn get_moderation_overview(
        &self,
        ctx: &Context<'_>,
        board_id: Option<i32>,
    ) -> Result<ModerationStats> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        // Check if user is admin or moderator
        let is_admin = user.has_permission(AdminPerms::Content);

        if !is_admin && board_id.is_none() {
            return Err(TinyBoardsError::from_message(
                403,
                "You must specify a board_id if you're not an admin",
            )
            .into());
        }

        // If board_id is specified, check if user is a moderator of that board
        if let Some(board_id) = board_id {
            if !is_admin {
                match BoardModerator::get_by_user_id_for_board(pool, user.id, board_id, true).await {
                    Ok(moderator) => {
                        if !moderator.has_permission(ModPerms::Content) {
                            return Err(TinyBoardsError::from_message(
                                403,
                                "You don't have permission to view moderation overview for this board",
                            )
                            .into());
                        }
                    }
                    Err(_) => {
                        return Err(TinyBoardsError::from_message(
                            403,
                            "You are not a moderator of this board",
                        )
                        .into());
                    }
                }
            }
        }

        let stats = ModerationLog::get_stats(pool, board_id).await?;
        Ok(ModerationStats::from(stats))
    }
}