use diesel::{prelude::*, Queryable, Insertable, QueryDsl, RunQueryDsl, ExpressionMethods};
use serde::{Deserialize, Serialize};
use crate::{
    schema::moderation_log,
    traits::Crud,
    utils::{DbPool, get_conn},
};
use tinyboards_utils::TinyBoardsError;

#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = moderation_log)]
pub struct ModerationLog {
    pub id: i32,
    pub moderator_id: i32,
    pub action_type: String,
    pub target_type: String,
    pub target_id: i32,
    pub board_id: Option<i32>,
    pub reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub creation_date: chrono::NaiveDateTime,
    pub expires_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Insertable, AsChangeset, Default)]
#[diesel(table_name = moderation_log)]
pub struct ModerationLogForm {
    pub moderator_id: Option<i32>,
    pub action_type: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<i32>,
    pub board_id: Option<i32>,
    pub reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub creation_date: Option<chrono::NaiveDateTime>,
    pub expires_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModerationStats {
    pub total_actions: i64,
    pub actions_today: i64,
    pub actions_this_week: i64,
    pub pending_reports: i64,
    pub banned_users: i64,
}

// Action types for consistent usage
pub mod action_types {
    pub const BAN_USER: &str = "ban_user";
    pub const UNBAN_USER: &str = "unban_user";
    pub const RESOLVE_REPORT: &str = "resolve_report";
    pub const APPROVE_POST: &str = "approve_post";
    pub const APPROVE_COMMENT: &str = "approve_comment";
    pub const REMOVE_POST: &str = "remove_post";
    pub const REMOVE_COMMENT: &str = "remove_comment";
    pub const LOCK_POST: &str = "lock_post";
    pub const UNLOCK_POST: &str = "unlock_post";
    pub const PIN_POST: &str = "pin_post";
    pub const UNPIN_POST: &str = "unpin_post";
    pub const BAN_FROM_BOARD: &str = "ban_from_board";
    pub const UNBAN_FROM_BOARD: &str = "unban_from_board";
}

// Target types for consistent usage
pub mod target_types {
    pub const USER: &str = "user";
    pub const POST: &str = "post";
    pub const COMMENT: &str = "comment";
    pub const REPORT: &str = "report";
    pub const BOARD: &str = "board";
}

#[async_trait::async_trait]
impl Crud for ModerationLog {
    type Form = ModerationLogForm;
    type IdType = i32;

    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, diesel::result::Error> {
        let conn = &mut get_conn(pool).await?;

        diesel_async::RunQueryDsl::get_result(
            diesel::insert_into(moderation_log::table)
                .values(form),
            conn
        )
        .await
    }

    async fn read(pool: &DbPool, id_: Self::IdType) -> Result<Self, diesel::result::Error> {
        let conn = &mut get_conn(pool).await?;

        diesel_async::RunQueryDsl::first(
            moderation_log::table.find(id_),
            conn
        )
        .await
    }

    async fn update(
        pool: &DbPool,
        id_: Self::IdType,
        form: &Self::Form,
    ) -> Result<Self, diesel::result::Error> {
        let conn = &mut get_conn(pool).await?;

        diesel_async::RunQueryDsl::get_result(
            diesel::update(moderation_log::table.find(id_))
                .set(form),
            conn
        )
        .await
    }

    async fn delete(pool: &DbPool, id_: Self::IdType) -> Result<usize, diesel::result::Error> {
        let conn = &mut get_conn(pool).await?;

        diesel_async::RunQueryDsl::execute(
            diesel::delete(moderation_log::table.find(id_)),
            conn
        )
        .await
    }
}

impl ModerationLog {
    /// Get moderation log entries with filtering
    pub async fn list(
        pool: &DbPool,
        board_id: Option<i32>,
        action_type: Option<String>,
        moderator_id: Option<i32>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<Self>, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        let mut query = moderation_log::table.into_boxed();

        if let Some(board_id) = board_id {
            query = query.filter(moderation_log::board_id.eq(board_id));
        }

        if let Some(action_type) = action_type {
            query = query.filter(moderation_log::action_type.eq(action_type));
        }

        if let Some(moderator_id) = moderator_id {
            query = query.filter(moderation_log::moderator_id.eq(moderator_id));
        }

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        use diesel_async::RunQueryDsl;

        let logs = diesel_async::RunQueryDsl::load(
            query
                .order(moderation_log::creation_date.desc())
                .limit(limit as i64)
                .offset(offset as i64),
            conn
        )
        .await
            .map_err(|e| TinyBoardsError::from(e))?;

        Ok(logs)
    }

    /// Log a moderation action
    pub async fn log_action(
        pool: &DbPool,
        moderator_id: i32,
        action_type: &str,
        target_type: &str,
        target_id: i32,
        board_id: Option<i32>,
        reason: Option<String>,
        metadata: Option<serde_json::Value>,
        expires_at: Option<chrono::NaiveDateTime>,
    ) -> Result<Self, TinyBoardsError> {
        let form = ModerationLogForm {
            moderator_id: Some(moderator_id),
            action_type: Some(action_type.to_string()),
            target_type: Some(target_type.to_string()),
            target_id: Some(target_id),
            board_id,
            reason,
            metadata,
            creation_date: Some(chrono::Utc::now().naive_utc()),
            expires_at,
        };

        Self::create(pool, &form).await.map_err(|e| TinyBoardsError::from(e))
    }

    /// Get moderation statistics
    pub async fn get_stats(
        pool: &DbPool,
        board_id: Option<i32>,
    ) -> Result<ModerationStats, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use diesel_async::RunQueryDsl;

        let now = chrono::Utc::now().naive_utc();
        let today = now.date().and_hms_opt(0, 0, 0).unwrap();
        let week_ago = today - chrono::Duration::days(7);

        let mut query = moderation_log::table.into_boxed();
        if let Some(board_id) = board_id {
            query = query.filter(moderation_log::board_id.eq(board_id));
        }

        let total_actions: i64 = diesel_async::RunQueryDsl::get_result(
            query.count(),
            conn
        )
        .await
            .map_err(|e| TinyBoardsError::from(e))?;

        let mut today_query = moderation_log::table.into_boxed();
        if let Some(board_id) = board_id {
            today_query = today_query.filter(moderation_log::board_id.eq(board_id));
        }

        let actions_today: i64 = diesel_async::RunQueryDsl::get_result(
            today_query
                .filter(moderation_log::creation_date.ge(today))
                .count(),
            conn
        )
        .await
            .map_err(|e| TinyBoardsError::from(e))?;

        let mut week_query = moderation_log::table.into_boxed();
        if let Some(board_id) = board_id {
            week_query = week_query.filter(moderation_log::board_id.eq(board_id));
        }

        let actions_this_week: i64 = diesel_async::RunQueryDsl::get_result(
            week_query
                .filter(moderation_log::creation_date.ge(week_ago))
                .count(),
            conn
        )
        .await
            .map_err(|e| TinyBoardsError::from(e))?;

        // Get pending reports count
        use crate::schema::{post_report, comment_report};

        let pending_post_reports: i64 = if let Some(board_id) = board_id {
            use crate::schema::posts;
            diesel_async::RunQueryDsl::get_result(
                post_report::table
                    .inner_join(posts::table.on(post_report::post_id.eq(posts::id)))
                    .filter(posts::board_id.eq(board_id))
                    .filter(post_report::resolved.eq(false))
                    .count(),
                conn
            )
            .await
            .map_err(|e| TinyBoardsError::from(e))?
        } else {
            diesel_async::RunQueryDsl::get_result(
                post_report::table
                    .filter(post_report::resolved.eq(false))
                    .count(),
                conn
            )
            .await
            .map_err(|e| TinyBoardsError::from(e))?
        };

        let pending_comment_reports: i64 = if let Some(board_id) = board_id {
            use crate::schema::comments;
            diesel_async::RunQueryDsl::get_result(
                comment_report::table
                    .inner_join(comments::table.on(comment_report::comment_id.eq(comments::id)))
                    .filter(comments::board_id.eq(board_id))
                    .filter(comment_report::resolved.eq(false))
                    .count(),
                conn
            )
            .await
            .map_err(|e| TinyBoardsError::from(e))?
        } else {
            diesel_async::RunQueryDsl::get_result(
                comment_report::table
                    .filter(comment_report::resolved.eq(false))
                    .count(),
                conn
            )
            .await
            .map_err(|e| TinyBoardsError::from(e))?
        };

        let pending_reports = pending_post_reports + pending_comment_reports;

        // Get banned users count
        use crate::schema::user_ban;

        let banned_users: i64 = diesel_async::RunQueryDsl::get_result(
            user_ban::table
                .filter(
                    user_ban::expires_at.is_null()
                        .or(user_ban::expires_at.gt(now))
                )
                .count(),
            conn
        )
        .await?;

        Ok(ModerationStats {
            total_actions,
            actions_today,
            actions_this_week,
            pending_reports,
            banned_users,
        })
    }
}