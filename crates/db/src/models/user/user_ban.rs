use crate::schema::user_ban;
use crate::utils::{DbPool, get_conn};
use crate::traits::Crud;
use tinyboards_utils::TinyBoardsError;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_ban)]
pub struct UserBan {
    pub id: i32,
    pub user_id: i32,
    pub creation_date: NaiveDateTime,
    pub banned_by: Option<i32>,
    pub reason: Option<String>,
    pub expires_at: Option<NaiveDateTime>,
    pub banned_at: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = user_ban)]
pub struct UserBanForm {
    pub user_id: Option<i32>,
    pub creation_date: Option<NaiveDateTime>,
    pub banned_by: Option<i32>,
    pub reason: Option<String>,
    pub expires_at: Option<NaiveDateTime>,
    pub banned_at: Option<NaiveDateTime>,
}


impl UserBan {
    /// Check if a user is currently banned
    pub async fn is_user_banned(pool: &DbPool, user_id: i32) -> Result<bool, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        let now = chrono::Utc::now().naive_utc();

        let ban_count: i64 = diesel_async::RunQueryDsl::get_result(
            user_ban::table
                .filter(user_ban::user_id.eq(user_id))
                .filter(
                    user_ban::expires_at.is_null()
                        .or(user_ban::expires_at.gt(now))
                )
                .count(),
            conn
        )
        .await?;

        Ok(ban_count > 0)
    }

    /// Get active ban for a user
    pub async fn get_active_ban(pool: &DbPool, user_id: i32) -> Result<Option<Self>, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        let now = chrono::Utc::now().naive_utc();

        let ban = diesel_async::RunQueryDsl::first(
            user_ban::table
                .filter(user_ban::user_id.eq(user_id))
                .filter(
                    user_ban::expires_at.is_null()
                        .or(user_ban::expires_at.gt(now))
                ),
            conn
        )
        .await
        .optional()?;

        Ok(ban)
    }


    /// Ban a user
    pub async fn ban_user(
        pool: &DbPool,
        user_id: i32,
        banned_by: i32,
        reason: Option<String>,
        expires_at: Option<NaiveDateTime>,
    ) -> Result<Self, TinyBoardsError> {
        let now = chrono::Utc::now().naive_utc();

        let form = UserBanForm {
            user_id: Some(user_id),
            creation_date: Some(now),
            banned_by: Some(banned_by),
            reason,
            expires_at,
            banned_at: Some(now),
        };

        Self::create(pool, &form).await.map_err(|e| TinyBoardsError::from(e))
    }

    /// List banned users with pagination
    pub async fn list_banned_users(
        pool: &DbPool,
        limit: Option<i32>,
        offset: Option<i32>,
        include_expired: bool,
    ) -> Result<Vec<Self>, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        let mut query = user_ban::table.into_boxed();

        if !include_expired {
            let now = chrono::Utc::now().naive_utc();
            query = query.filter(
                user_ban::expires_at.is_null()
                    .or(user_ban::expires_at.gt(now))
            );
        }

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let bans = diesel_async::RunQueryDsl::load(
            query
                .order(user_ban::banned_at.desc())
                .limit(limit as i64)
                .offset(offset as i64),
            conn
        )
        .await?;

        Ok(bans)
    }
}