use crate::{
    models::flair::flair_aggregates::FlairAggregates,
    schema::{flair_aggregates, flair_templates, post_flairs, user_flairs},
    utils::{get_conn, DbPool},
};
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

impl FlairAggregates {
    /// Get aggregates for a flair template
    pub async fn get_by_template(
        pool: &DbPool,
        template_id: i32,
    ) -> Result<Option<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        flair_aggregates::table
            .filter(flair_aggregates::flair_template_id.eq(template_id))
            .first::<Self>(conn)
            .await
            .optional()
    }

    /// Get aggregates for all templates in a board
    pub async fn get_for_board(
        pool: &DbPool,
        board_id: i32,
    ) -> Result<Vec<(crate::models::flair::FlairTemplate, Self)>, Error> {
        let conn = &mut get_conn(pool).await?;
        flair_templates::table
            .inner_join(flair_aggregates::table.on(flair_aggregates::flair_template_id.eq(flair_templates::id)))
            .filter(flair_templates::board_id.eq(board_id))
            .filter(flair_templates::is_active.eq(true))
            .select((flair_templates::all_columns, flair_aggregates::all_columns))
            .load::<(crate::models::flair::FlairTemplate, Self)>(conn)
            .await
    }

    /// Refresh aggregates for a flair template
    pub async fn refresh(pool: &DbPool, template_id: i32) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        // Count post flairs
        let post_count = post_flairs::table
            .filter(post_flairs::flair_template_id.eq(template_id))
            .count()
            .get_result::<i64>(conn)
            .await
            .unwrap_or(0);

        // Count user flairs (only approved)
        let user_count = user_flairs::table
            .filter(user_flairs::flair_template_id.eq(template_id))
            .filter(user_flairs::is_approved.eq(true))
            .count()
            .get_result::<i64>(conn)
            .await
            .unwrap_or(0);

        // Get last used date (max of post and user flair assignment dates)
        let last_post_date = post_flairs::table
            .filter(post_flairs::flair_template_id.eq(template_id))
            .select(diesel::dsl::max(post_flairs::assigned_at))
            .first::<Option<chrono::NaiveDateTime>>(conn)
            .await
            .ok()
            .flatten();

        let last_user_date = user_flairs::table
            .filter(user_flairs::flair_template_id.eq(template_id))
            .filter(user_flairs::is_approved.eq(true))
            .select(diesel::dsl::max(user_flairs::assigned_at))
            .first::<Option<chrono::NaiveDateTime>>(conn)
            .await
            .ok()
            .flatten();

        let last_used = match (last_post_date, last_user_date) {
            (Some(p), Some(u)) => Some(if p > u { p } else { u }),
            (Some(p), None) => Some(p),
            (None, Some(u)) => Some(u),
            (None, None) => None,
        };

        let total_uses = post_count + user_count;

        // Update or create aggregates
        let existing = Self::get_by_template(pool, template_id).await?;

        if let Some(existing) = existing {
            diesel::update(flair_aggregates::table.find(existing.id))
                .set((
                    flair_aggregates::total_usage_count.eq(total_uses as i32),
                    flair_aggregates::post_usage_count.eq(post_count as i32),
                    flair_aggregates::user_usage_count.eq(user_count as i32),
                    flair_aggregates::last_used_at.eq(last_used),
                ))
                .get_result::<Self>(conn)
                .await
                .map_err(|e| {
                    TinyBoardsError::from_error_message(
                        e,
                        500,
                        "Failed to update flair aggregates",
                    )
                })
        } else {
            diesel::insert_into(flair_aggregates::table)
                .values((
                    flair_aggregates::flair_template_id.eq(template_id),
                    flair_aggregates::total_usage_count.eq(total_uses as i32),
                    flair_aggregates::post_usage_count.eq(post_count as i32),
                    flair_aggregates::user_usage_count.eq(user_count as i32),
                    flair_aggregates::last_used_at.eq(last_used),
                ))
                .get_result::<Self>(conn)
                .await
                .map_err(|e| {
                    TinyBoardsError::from_error_message(
                        e,
                        500,
                        "Failed to create flair aggregates",
                    )
                })
        }
    }

    /// Refresh aggregates for all templates in a board
    pub async fn refresh_for_board(
        pool: &DbPool,
        board_id: i32,
    ) -> Result<Vec<Self>, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        // Get all template IDs for the board
        let template_ids: Vec<i32> = flair_templates::table
            .filter(flair_templates::board_id.eq(board_id))
            .filter(flair_templates::is_active.eq(true))
            .select(flair_templates::id)
            .load::<i32>(conn)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to get flair templates")
            })?;

        // Refresh aggregates for each template
        let mut results = Vec::new();
        for template_id in template_ids {
            let agg = Self::refresh(pool, template_id).await?;
            results.push(agg);
        }

        Ok(results)
    }

    /// Increment usage count when a flair is used
    pub async fn increment_usage(
        pool: &DbPool,
        template_id: i32,
        is_post: bool,
    ) -> Result<(), TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        let existing = Self::get_by_template(pool, template_id).await?;

        let now = chrono::Utc::now().naive_utc();

        if let Some(existing) = existing {
            if is_post {
                diesel::update(flair_aggregates::table.find(existing.id))
                    .set((
                        flair_aggregates::total_usage_count.eq(flair_aggregates::total_usage_count + 1),
                        flair_aggregates::post_usage_count.eq(flair_aggregates::post_usage_count + 1),
                        flair_aggregates::last_used_at.eq(now),
                    ))
                    .execute(conn)
                    .await
                    .map(|_| ())
                    .map_err(|e| {
                        TinyBoardsError::from_error_message(
                            e,
                            500,
                            "Failed to increment flair usage",
                        )
                    })
            } else {
                diesel::update(flair_aggregates::table.find(existing.id))
                    .set((
                        flair_aggregates::total_usage_count.eq(flair_aggregates::total_usage_count + 1),
                        flair_aggregates::user_usage_count.eq(flair_aggregates::user_usage_count + 1),
                        flair_aggregates::last_used_at.eq(now),
                    ))
                    .execute(conn)
                    .await
                    .map(|_| ())
                    .map_err(|e| {
                        TinyBoardsError::from_error_message(
                            e,
                            500,
                            "Failed to increment flair usage",
                        )
                    })
            }
        } else {
            // Create initial aggregates
            diesel::insert_into(flair_aggregates::table)
                .values((
                    flair_aggregates::flair_template_id.eq(template_id),
                    flair_aggregates::total_usage_count.eq(1i32),
                    flair_aggregates::post_usage_count.eq(if is_post { 1i32 } else { 0i32 }),
                    flair_aggregates::user_usage_count.eq(if is_post { 0i32 } else { 1i32 }),
                    flair_aggregates::last_used_at.eq(now),
                ))
                .execute(conn)
                .await
                .map(|_| ())
                .map_err(|e| {
                    TinyBoardsError::from_error_message(e, 500, "Failed to create flair aggregates")
                })
        }
    }
}
