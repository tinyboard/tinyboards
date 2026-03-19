use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::user::user::User as DbUser,
    models::aggregates::UserAggregates as DbUserAggregates,
    schema::{users, user_aggregates, user_follows},
    utils::{get_conn, DbPool, fuzzy_search, limit_and_offset},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{
    helpers::{permissions, validation::check_private_instance},
    structs::user::{User, UserSettings},
};

#[derive(Default)]
pub struct QueryUser;

#[Object]
impl QueryUser {
    /// Get user by username (public).
    pub async fn user(&self, ctx: &Context<'_>, username: String) -> Result<User> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = permissions::optional_auth(ctx);
        check_private_instance(v_opt, pool).await?;

        let conn = &mut get_conn(pool).await?;

        let db_user: DbUser = users::table
            .filter(users::name.eq(&username))
            .filter(users::deleted_at.is_null())
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("User not found".to_string()))?;

        let agg: Option<DbUserAggregates> = user_aggregates::table
            .filter(user_aggregates::user_id.eq(db_user.id))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(User::from_db(db_user, agg))
    }

    /// List users with search and filtering.
    pub async fn list_users(
        &self,
        ctx: &Context<'_>,
        search_term: Option<String>,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<User>> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = permissions::optional_auth(ctx);
        check_private_instance(v_opt, pool).await?;

        let conn = &mut get_conn(pool).await?;
        let (limit_val, offset_val) = limit_and_offset(page, limit)
            .map_err(|e| TinyBoardsError::BadRequest(e.to_string()))?;

        let mut query = users::table
            .filter(users::deleted_at.is_null())
            .filter(users::is_banned.eq(false))
            .into_boxed();

        if let Some(ref term) = search_term {
            query = query.filter(users::name.ilike(fuzzy_search(term)));
        }

        let db_users: Vec<DbUser> = query
            .order(users::created_at.desc())
            .limit(limit_val)
            .offset(offset_val)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Batch load aggregates
        let user_ids: Vec<Uuid> = db_users.iter().map(|u| u.id).collect();
        let aggs: Vec<DbUserAggregates> = user_aggregates::table
            .filter(user_aggregates::user_id.eq_any(&user_ids))
            .load(conn)
            .await
            .unwrap_or_default();

        let result = db_users.into_iter().map(|u| {
            let agg = aggs.iter().find(|a| a.user_id == u.id).cloned();
            User::from_db(u, agg)
        }).collect();

        Ok(result)
    }

    /// Username autocomplete search.
    pub async fn search_usernames(
        &self,
        ctx: &Context<'_>,
        query: String,
        limit: Option<i64>,
    ) -> Result<Vec<String>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let limit = limit.unwrap_or(10).min(25);

        let names: Vec<String> = users::table
            .select(users::name)
            .filter(users::name.ilike(format!("{}%", query)))
            .filter(users::deleted_at.is_null())
            .limit(limit)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(names)
    }

    /// Get the authenticated user's settings (private).
    pub async fn get_user_settings(&self, ctx: &Context<'_>) -> Result<UserSettings> {
        let user = permissions::require_auth(ctx)?;
        Ok(UserSettings::from(user.clone()))
    }

    /// Get followers of a user.
    pub async fn user_followers(&self, ctx: &Context<'_>, user_id: ID) -> Result<Vec<User>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let uid: Uuid = user_id.parse().map_err(|_| TinyBoardsError::BadRequest("Invalid UUID".to_string()))?;

        let follower_ids: Vec<Uuid> = user_follows::table
            .filter(user_follows::user_id.eq(uid))
            .filter(user_follows::is_pending.eq(false))
            .select(user_follows::follower_id)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let followers: Vec<DbUser> = users::table
            .filter(users::id.eq_any(&follower_ids))
            .filter(users::deleted_at.is_null())
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(followers.into_iter().map(|u| User::from_db(u, None)).collect())
    }

    /// Get users that a user is following.
    pub async fn user_following(&self, ctx: &Context<'_>, user_id: ID) -> Result<Vec<User>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let uid: Uuid = user_id.parse().map_err(|_| TinyBoardsError::BadRequest("Invalid UUID".to_string()))?;

        let following_ids: Vec<Uuid> = user_follows::table
            .filter(user_follows::follower_id.eq(uid))
            .filter(user_follows::is_pending.eq(false))
            .select(user_follows::user_id)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let following: Vec<DbUser> = users::table
            .filter(users::id.eq_any(&following_ids))
            .filter(users::deleted_at.is_null())
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(following.into_iter().map(|u| User::from_db(u, None)).collect())
    }

    /// Check if current user is following another user.
    pub async fn is_following_user(&self, ctx: &Context<'_>, user_id: ID) -> Result<bool> {
        let me = permissions::require_auth(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let uid: Uuid = user_id.parse().map_err(|_| TinyBoardsError::BadRequest("Invalid UUID".to_string()))?;

        let exists: bool = diesel::dsl::select(diesel::dsl::exists(
            user_follows::table
                .filter(user_follows::user_id.eq(uid))
                .filter(user_follows::follower_id.eq(me.id))
        ))
        .get_result(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(exists)
    }
}
