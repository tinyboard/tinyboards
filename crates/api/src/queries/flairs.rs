use crate::{
    LoggedInUser,
    structs::flair::{
        FlairTemplate, PostFlair, UserFlair, FlairCategory, FlairType
    },
};
use async_graphql::*;
use tinyboards_db::utils::DbPool;

#[derive(Default)]
pub struct FlairQueries;

#[Object]
impl FlairQueries {
    /// Get flair templates for a specific board
    /// Returns both board-specific and site-wide templates if board_id is provided
    async fn board_flairs(
        &self,
        ctx: &Context<'_>,
        board_id: Option<i32>,
        flair_type: Option<FlairType>,
        active_only: Option<bool>,
    ) -> Result<Vec<FlairTemplate>> {
        let pool = ctx.data::<DbPool>()?;

        if let Some(board_id) = board_id {
            let flair_type_str = flair_type.map(|ft| ft.as_str());
            let templates = tinyboards_db::models::flair::FlairTemplate::get_by_board(
                pool,
                board_id,
                flair_type_str
            ).await?;

            Ok(templates.into_iter().map(FlairTemplate::from).collect())
        } else {
            // No board specified, return empty list (site-wide flairs handled by site_flairs)
            Ok(vec![])
        }
    }

    /// Get site-wide flair templates (not board-specific)
    async fn site_flairs(
        &self,
        ctx: &Context<'_>,
        _flair_type: Option<FlairType>,
        _active_only: Option<bool>,
    ) -> Result<Vec<FlairTemplate>> {
        let _pool = ctx.data::<DbPool>()?;
        // Site-wide flairs are not currently implemented in the database schema
        // All flairs are board-specific
        Ok(vec![])
    }

    /// Get a specific flair template by ID
    async fn flair_template(
        &self,
        ctx: &Context<'_>,
        template_id: i32,
    ) -> Result<Option<FlairTemplate>> {
        let pool = ctx.data::<DbPool>()?;

        use tinyboards_db::traits::Crud;
        match tinyboards_db::models::flair::FlairTemplate::read(pool, template_id).await {
            Ok(template) => Ok(Some(FlairTemplate::from(template))),
            Err(_) => Ok(None),
        }
    }

    /// Get a specific flair template by ID (alias for flair_template)
    async fn flair(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> Result<Option<FlairTemplate>> {
        self.flair_template(ctx, id).await
    }

    /// Get usage statistics for a flair template
    async fn get_flair_usage_stats(
        &self,
        ctx: &Context<'_>,
        flair_id: i32,
    ) -> Result<FlairUsageStats> {
        let pool = ctx.data::<DbPool>()?;

        use tinyboards_db::traits::Crud;
        let template = tinyboards_db::models::flair::FlairTemplate::read(pool, flair_id).await?;

        // For now, return basic stats from the template
        // TODO: Implement actual statistics queries
        Ok(FlairUsageStats {
            total_usage: template.usage_count as i64,
            this_week: 0,
            this_month: 0,
            unique_users: 0,
        })
    }

    /// Get usage chart data for a flair template
    async fn get_flair_usage_chart(
        &self,
        _ctx: &Context<'_>,
        _flair_id: i32,
    ) -> Result<Vec<FlairUsageChartData>> {
        // TODO: Implement actual chart data queries
        Ok(vec![])
    }

    /// Get posts using a specific flair
    async fn get_flair_posts(
        &self,
        ctx: &Context<'_>,
        flair_id: i32,
        limit: Option<i32>,
    ) -> Result<Vec<crate::structs::post::Post>> {
        let pool = ctx.data::<DbPool>()?;
        let limit = limit.unwrap_or(50).min(100) as i64;

        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use tinyboards_db::{
            schema::{post_flairs, posts, post_aggregates},
            utils::get_conn,
            models::post::posts::Post as DbPost,
            aggregates::structs::PostAggregates as DbPostAggregates
        };

        let conn = &mut get_conn(pool).await?;

        let results: Vec<(DbPost, DbPostAggregates)> = post_flairs::table
            .inner_join(posts::table.on(post_flairs::post_id.eq(posts::id)))
            .inner_join(post_aggregates::table.on(posts::id.eq(post_aggregates::post_id)))
            .filter(post_flairs::flair_template_id.eq(flair_id))
            .select((posts::all_columns, post_aggregates::all_columns))
            .limit(limit)
            .load::<(DbPost, DbPostAggregates)>(conn)
            .await?;

        Ok(results.into_iter().map(crate::structs::post::Post::from).collect())
    }

    /// Get users with a specific flair
    async fn get_flair_users(
        &self,
        ctx: &Context<'_>,
        flair_id: i32,
        limit: Option<i32>,
    ) -> Result<Vec<crate::structs::user::User>> {
        let pool = ctx.data::<DbPool>()?;
        let limit = limit.unwrap_or(50).min(100) as i64;

        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use tinyboards_db::{schema::user_flairs, utils::get_conn, traits::Crud};

        let conn = &mut get_conn(pool).await?;

        let user_ids: Vec<i32> = user_flairs::table
            .filter(user_flairs::flair_template_id.eq(flair_id))
            .filter(user_flairs::is_approved.eq(true))
            .select(user_flairs::user_id)
            .limit(limit)
            .load::<i32>(conn)
            .await?;

        let mut result_users = vec![];
        for user_id in user_ids {
            if let Ok(user) = tinyboards_db::models::user::user::User::read(pool, user_id).await {
                result_users.push(crate::structs::user::User::from(user));
            }
        }

        Ok(result_users)
    }

    /// Get top posts with a specific flair (by score)
    async fn get_flair_top_posts(
        &self,
        ctx: &Context<'_>,
        flair_id: i32,
        limit: Option<i32>,
    ) -> Result<Vec<crate::structs::post::Post>> {
        let pool = ctx.data::<DbPool>()?;
        let limit = limit.unwrap_or(5).min(20) as i64;

        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use tinyboards_db::{
            schema::{posts, post_flairs, post_aggregates},
            utils::get_conn,
            models::post::posts::Post as DbPost,
            aggregates::structs::PostAggregates as DbPostAggregates
        };

        let conn = &mut get_conn(pool).await?;

        let results: Vec<(DbPost, DbPostAggregates)> = post_flairs::table
            .inner_join(posts::table.on(post_flairs::post_id.eq(posts::id)))
            .inner_join(post_aggregates::table.on(posts::id.eq(post_aggregates::post_id)))
            .filter(post_flairs::flair_template_id.eq(flair_id))
            .select((posts::all_columns, post_aggregates::all_columns))
            .order(post_aggregates::score.desc())
            .limit(limit)
            .load::<(DbPost, DbPostAggregates)>(conn)
            .await?;

        Ok(results.into_iter().map(crate::structs::post::Post::from).collect())
    }

    /// Get top users with a specific flair (by combined post and comment score)
    async fn get_flair_top_users(
        &self,
        ctx: &Context<'_>,
        flair_id: i32,
        limit: Option<i32>,
    ) -> Result<Vec<crate::structs::user::User>> {
        let pool = ctx.data::<DbPool>()?;
        let limit = limit.unwrap_or(5).min(20) as i64;

        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use tinyboards_db::{schema::{user_flairs, users, user_aggregates}, utils::get_conn, traits::Crud};

        let conn = &mut get_conn(pool).await?;

        let user_ids: Vec<i32> = user_flairs::table
            .inner_join(users::table.on(user_flairs::user_id.eq(users::id)))
            .inner_join(user_aggregates::table.on(users::id.eq(user_aggregates::user_id)))
            .filter(user_flairs::flair_template_id.eq(flair_id))
            .filter(user_flairs::is_approved.eq(true))
            .select(user_flairs::user_id)
            .order((user_aggregates::post_score + user_aggregates::comment_score).desc())
            .limit(limit)
            .load::<i32>(conn)
            .await?;

        let mut result_users = vec![];
        for user_id in user_ids {
            if let Ok(user) = tinyboards_db::models::user::user::User::read(pool, user_id).await {
                result_users.push(crate::structs::user::User::from(user));
            }
        }

        Ok(result_users)
    }

    /// Get flair assigned to a specific post
    async fn post_flair(
        &self,
        ctx: &Context<'_>,
        post_id: i32,
    ) -> Result<Option<PostFlair>> {
        let pool = ctx.data::<DbPool>()?;

        match tinyboards_db::models::flair::PostFlair::get_by_post(pool, post_id).await {
            Ok(Some(flair)) => Ok(Some(PostFlair::from(flair))),
            Ok(None) | Err(_) => Ok(None),
        }
    }

    /// Get flair assigned to a specific user (optionally for a specific board)
    async fn user_flair(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
        board_id: Option<i32>,
    ) -> Result<Option<UserFlair>> {
        let pool = ctx.data::<DbPool>()?;

        if let Some(board_id) = board_id {
            match tinyboards_db::models::flair::UserFlair::get_by_user_and_board(pool, user_id, board_id).await {
                Ok(Some(flair)) => Ok(Some(UserFlair::from(flair))),
                Ok(None) | Err(_) => Ok(None),
            }
        } else {
            // Without board_id, we can't retrieve user flair (it's board-specific)
            Ok(None)
        }
    }

    /// Get all flair categories for a board
    async fn flair_categories(
        &self,
        ctx: &Context<'_>,
        board_id: i32,
    ) -> Result<Vec<FlairCategory>> {
        let pool = ctx.data::<DbPool>()?;

        use tinyboards_db::models::flair::FlairCategory as DbFlairCategory;

        let categories = DbFlairCategory::for_board(pool, board_id).await?;

        Ok(categories.into_iter().map(FlairCategory::from).collect())
    }

    /// Get all flairs for a board (admin/mod view with inactive ones)
    async fn manage_board_flairs(
        &self,
        ctx: &Context<'_>,
        board_id: i32,
        flair_type: Option<FlairType>,
    ) -> Result<Vec<FlairTemplate>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        use tinyboards_db::models::{user::user::AdminPerms, board::board_mods::ModPerms};

        // Check permissions - must be mod or admin
        crate::helpers::validation::require_mod_or_admin(
            user,
            pool,
            board_id,
            ModPerms::Flair,
            Some(AdminPerms::Flair),
        ).await?;

        let flair_type_str = flair_type.map(|ft| ft.as_str());
        let templates = tinyboards_db::models::flair::FlairTemplate::get_by_board(
            pool,
            board_id,
            flair_type_str
        ).await?;

        Ok(templates.into_iter().map(FlairTemplate::from).collect())
    }

    /// Get pending user flair assignments for approval (mod only)
    async fn pending_user_flairs(
        &self,
        ctx: &Context<'_>,
        board_id: i32,
    ) -> Result<Vec<UserFlair>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        use tinyboards_db::models::{user::user::AdminPerms, board::board_mods::ModPerms};

        // Check permissions - must be mod or admin
        crate::helpers::validation::require_mod_or_admin(
            user,
            pool,
            board_id,
            ModPerms::Flair,
            Some(AdminPerms::Flair),
        ).await?;

        let pending_flairs = tinyboards_db::models::flair::UserFlair::get_pending_for_board(
            pool,
            board_id,
            None,
            None
        ).await?;

        Ok(pending_flairs.into_iter().map(UserFlair::from).collect())
    }

    /// Get user's flair filter preferences
    async fn my_flair_filters(
        &self,
        ctx: &Context<'_>,
        board_id: Option<i32>,
    ) -> Result<Option<FlairFilters>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user()?;

        if let Some(board_id) = board_id {
            match tinyboards_db::models::flair::UserFlairFilter::get_by_user_and_board(pool, user.id, board_id).await {
                Ok(Some(filter)) => {
                    // Convert database model to GraphQL type
                    let hidden_flair_ids = filter.excluded_flair_ids
                        .into_iter()
                        .filter_map(|id| id)
                        .collect();

                    let highlighted_flair_ids = filter.included_flair_ids
                        .into_iter()
                        .filter_map(|id| id)
                        .collect();

                    Ok(Some(FlairFilters {
                        user_id: filter.user_id,
                        board_id: Some(filter.board_id),
                        hidden_flair_ids,
                        highlighted_flair_ids,
                    }))
                },
                Ok(None) | Err(_) => Ok(None),
            }
        } else {
            // Without board_id, we can't retrieve filters (they're board-specific)
            Ok(None)
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct FlairFilters {
    pub user_id: i32,
    pub board_id: Option<i32>,
    pub hidden_flair_ids: Vec<i32>,
    pub highlighted_flair_ids: Vec<i32>,
}

#[derive(SimpleObject, Clone)]
pub struct FlairUsageStats {
    pub total_usage: i64,
    pub this_week: i64,
    pub this_month: i64,
    pub unique_users: i64,
}

#[derive(SimpleObject, Clone)]
pub struct FlairUsageChartData {
    pub date: String,
    pub label: String,
    pub count: i64,
}
