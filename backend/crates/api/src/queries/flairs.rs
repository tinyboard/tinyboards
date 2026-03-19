use crate::{
    structs::flair::{FlairAggregatesView, FlairCategory, FlairTemplate, FlairType, PostFlair, UserFlair},
    LoggedInUser,
};
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbFlairType,
    models::{
        aggregates::FlairAggregates as DbFlairAggregates,
        flair::{
            FlairCategory as DbFlairCategory,
            FlairTemplate as DbFlairTemplate, PostFlair as DbPostFlair, UserFlair as DbUserFlair,
            UserFlairFilter as DbUserFlairFilter,
        },
    },
    schema::{flair_aggregates, flair_categories, flair_templates, post_flairs, user_flair_filters, user_flairs},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

#[derive(Default)]
pub struct FlairQueries;

#[Object]
impl FlairQueries {
    /// Get flair templates for a board, filtered by type and active status
    async fn board_flairs(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
        flair_type: Option<FlairType>,
        #[graphql(default = true)] active_only: bool,
    ) -> Result<Vec<FlairTemplate>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        let mut query = flair_templates::table
            .filter(flair_templates::board_id.eq(board_uuid))
            .into_boxed();

        if let Some(ft) = flair_type {
            let db_type = match ft {
                FlairType::Post => DbFlairType::Post,
                FlairType::User => DbFlairType::User,
            };
            query = query.filter(flair_templates::flair_type.eq(db_type));
        }

        if active_only {
            query = query.filter(flair_templates::is_active.eq(true));
        }

        let templates: Vec<DbFlairTemplate> = query
            .order(flair_templates::display_order.asc())
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(templates.into_iter().map(FlairTemplate::from).collect())
    }

    /// Get a specific flair template by ID
    async fn flair_template(&self, ctx: &Context<'_>, id: ID) -> Result<Option<FlairTemplate>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let flair_uuid: Uuid = id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid flair template ID".into()))?;

        let template: Option<DbFlairTemplate> = flair_templates::table
            .find(flair_uuid)
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(template.map(FlairTemplate::from))
    }

    /// Get flair usage statistics from flair_aggregates (BUG-037 fix: real stats, not stubs)
    async fn get_flair_usage_stats(
        &self,
        ctx: &Context<'_>,
        flair_id: ID,
    ) -> Result<Option<FlairAggregatesView>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let flair_uuid: Uuid = flair_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid flair ID".into()))?;

        let agg: Option<DbFlairAggregates> = flair_aggregates::table
            .filter(flair_aggregates::flair_template_id.eq(flair_uuid))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(agg.map(|a| FlairAggregatesView {
            flair_template_id: a.flair_template_id.to_string().into(),
            total_usage_count: a.total_usage_count,
            post_usage_count: a.post_usage_count,
            user_usage_count: a.user_usage_count,
            active_user_count: a.active_user_count,
            usage_last_day: a.usage_last_day,
            usage_last_week: a.usage_last_week,
            usage_last_month: a.usage_last_month,
            total_post_comments: a.total_post_comments,
            total_post_score: a.total_post_score,
            last_used_at: a.last_used_at.map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339()),
        }))
    }

    /// Get posts using a specific flair (batch query, fixes N+1)
    async fn get_flair_posts(
        &self,
        ctx: &Context<'_>,
        flair_id: ID,
        limit: Option<i32>,
    ) -> Result<Vec<PostFlair>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let limit = limit.unwrap_or(50).min(100) as i64;

        let flair_uuid: Uuid = flair_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid flair ID".into()))?;

        let results: Vec<DbPostFlair> = post_flairs::table
            .filter(post_flairs::flair_template_id.eq(flair_uuid))
            .order(post_flairs::created_at.desc())
            .limit(limit)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(results.into_iter().map(PostFlair::from).collect())
    }

    /// Get users with a specific flair (batch query, fixes N+1)
    async fn get_flair_users(
        &self,
        ctx: &Context<'_>,
        flair_id: ID,
        limit: Option<i32>,
    ) -> Result<Vec<UserFlair>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let limit = limit.unwrap_or(50).min(100) as i64;

        let flair_uuid: Uuid = flair_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid flair ID".into()))?;

        let results: Vec<DbUserFlair> = user_flairs::table
            .filter(user_flairs::flair_template_id.eq(flair_uuid))
            .filter(user_flairs::is_approved.eq(true))
            .order(user_flairs::created_at.desc())
            .limit(limit)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(results.into_iter().map(UserFlair::from).collect())
    }

    /// Get flairs assigned to a specific post
    async fn post_flair(&self, ctx: &Context<'_>, post_id: ID) -> Result<Vec<PostFlair>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = post_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid post ID".into()))?;

        let flairs: Vec<DbPostFlair> = post_flairs::table
            .filter(post_flairs::post_id.eq(post_uuid))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(flairs.into_iter().map(PostFlair::from).collect())
    }

    /// Get flair assigned to a user in a board
    async fn user_flair(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        board_id: ID,
    ) -> Result<Option<UserFlair>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let user_uuid: Uuid = user_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid user ID".into()))?;
        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        let flair: Option<DbUserFlair> = user_flairs::table
            .filter(user_flairs::user_id.eq(user_uuid))
            .filter(user_flairs::board_id.eq(board_uuid))
            .filter(user_flairs::is_approved.eq(true))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(flair.map(UserFlair::from))
    }

    /// Get flair categories for a board
    async fn flair_categories(&self, ctx: &Context<'_>, board_id: ID) -> Result<Vec<FlairCategory>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        let categories: Vec<DbFlairCategory> = flair_categories::table
            .filter(flair_categories::board_id.eq(board_uuid))
            .order(flair_categories::display_order.asc())
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(categories.into_iter().map(FlairCategory::from).collect())
    }

    /// Get all flairs for a board (mod view, includes inactive)
    async fn manage_board_flairs(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
        flair_type: Option<FlairType>,
    ) -> Result<Vec<FlairTemplate>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        // Check mod or admin permissions
        use tinyboards_db::{
            models::board::board_mods::{BoardModerator, ModPerms},
            schema::board_moderators,
        };

        if !user.is_admin {
            let moderator: BoardModerator = board_moderators::table
                .filter(board_moderators::board_id.eq(board_uuid))
                .filter(board_moderators::user_id.eq(user.id))
                .filter(board_moderators::is_invite_accepted.eq(true))
                .first(conn)
                .await
                .map_err(|_| {
                    TinyBoardsError::from_message(403, "You are not a moderator of this board")
                })?;

            if !moderator.has_permission(ModPerms::Content) {
                return Err(TinyBoardsError::from_message(
                    403,
                    "Insufficient permissions to manage flairs",
                )
                .into());
            }
        }

        let mut query = flair_templates::table
            .filter(flair_templates::board_id.eq(board_uuid))
            .into_boxed();

        if let Some(ft) = flair_type {
            let db_type = match ft {
                FlairType::Post => DbFlairType::Post,
                FlairType::User => DbFlairType::User,
            };
            query = query.filter(flair_templates::flair_type.eq(db_type));
        }

        let templates: Vec<DbFlairTemplate> = query
            .order(flair_templates::display_order.asc())
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(templates.into_iter().map(FlairTemplate::from).collect())
    }

    /// Get pending user flair assignments (mod only)
    async fn pending_user_flairs(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
    ) -> Result<Vec<UserFlair>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        // Check mod permissions
        use tinyboards_db::{models::board::board_mods::BoardModerator, schema::board_moderators};

        if !user.is_admin {
            let _moderator: BoardModerator = board_moderators::table
                .filter(board_moderators::board_id.eq(board_uuid))
                .filter(board_moderators::user_id.eq(user.id))
                .filter(board_moderators::is_invite_accepted.eq(true))
                .first(conn)
                .await
                .map_err(|_| {
                    TinyBoardsError::from_message(403, "You are not a moderator of this board")
                })?;
        }

        let pending: Vec<DbUserFlair> = user_flairs::table
            .filter(user_flairs::board_id.eq(board_uuid))
            .filter(user_flairs::is_approved.eq(false))
            .order(user_flairs::created_at.asc())
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(pending.into_iter().map(UserFlair::from).collect())
    }

    /// Get user's flair filter preferences for a board
    async fn my_flair_filters(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
    ) -> Result<Option<FlairFilterView>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user()?;
        let conn = &mut get_conn(pool).await?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        let filter: Option<DbUserFlairFilter> = user_flair_filters::table
            .filter(user_flair_filters::user_id.eq(user.id))
            .filter(user_flair_filters::board_id.eq(board_uuid))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(filter.map(|f| {
            let filter_mode_str = match f.filter_mode {
                tinyboards_db::enums::DbFilterMode::Include => "include",
                tinyboards_db::enums::DbFilterMode::Exclude => "exclude",
            };
            FlairFilterView {
                id: f.id.to_string().into(),
                board_id: f.board_id.to_string().into(),
                filter_mode: filter_mode_str.to_string(),
                included_flair_ids: f.included_flair_ids.into_iter().flatten().collect(),
                excluded_flair_ids: f.excluded_flair_ids.into_iter().flatten().collect(),
            }
        }))
    }
}

#[derive(SimpleObject, Clone)]
pub struct FlairFilterView {
    pub id: ID,
    pub board_id: ID,
    pub filter_mode: String,
    pub included_flair_ids: Vec<i32>,
    pub excluded_flair_ids: Vec<i32>,
}
