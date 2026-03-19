use crate::{
    queries::flairs::FlairFilterView,
    structs::flair::UpdateFlairFiltersInput,
    LoggedInUser,
};
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbFilterMode,
    models::flair::{UserFlairFilter as DbUserFlairFilter, UserFlairFilterInsertForm, UserFlairFilterUpdateForm},
    schema::user_flair_filters,
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

#[derive(Default)]
pub struct FlairFilterMutations;

#[Object]
impl FlairFilterMutations {
    /// Update user's flair filter preferences for a board (BUG-030 fix: proper Option handling)
    async fn update_flair_filters(
        &self,
        ctx: &Context<'_>,
        input: UpdateFlairFiltersInput,
    ) -> Result<FlairFilterView> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let board_uuid: Uuid = input
            .board_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        let db_filter_mode = match input.filter_mode {
            Some(crate::structs::flair::FilterMode::Include) => DbFilterMode::Include,
            Some(crate::structs::flair::FilterMode::Exclude) => DbFilterMode::Exclude,
            None => DbFilterMode::Exclude, // Default to exclude mode
        };

        let included = input
            .included_flair_ids
            .unwrap_or_default()
            .into_iter()
            .map(Some)
            .collect::<Vec<Option<i32>>>();

        let excluded = input
            .excluded_flair_ids
            .unwrap_or_default()
            .into_iter()
            .map(Some)
            .collect::<Vec<Option<i32>>>();

        // Check if filter already exists
        let existing: Option<DbUserFlairFilter> = user_flair_filters::table
            .filter(user_flair_filters::user_id.eq(user.id))
            .filter(user_flair_filters::board_id.eq(board_uuid))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let filter: DbUserFlairFilter = if let Some(existing_filter) = existing {
            let update_form = UserFlairFilterUpdateForm {
                filter_mode: Some(db_filter_mode),
                included_flair_ids: Some(included),
                excluded_flair_ids: Some(excluded),
            };

            diesel::update(user_flair_filters::table.find(existing_filter.id))
                .set(&update_form)
                .get_result(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?
        } else {
            let insert_form = UserFlairFilterInsertForm {
                user_id: user.id,
                board_id: board_uuid,
                filter_mode: db_filter_mode,
                included_flair_ids: included,
                excluded_flair_ids: excluded,
            };

            diesel::insert_into(user_flair_filters::table)
                .values(&insert_form)
                .get_result(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?
        };

        let filter_mode_str = match filter.filter_mode {
            DbFilterMode::Include => "include",
            DbFilterMode::Exclude => "exclude",
        };

        Ok(FlairFilterView {
            id: filter.id.to_string().into(),
            board_id: filter.board_id.to_string().into(),
            filter_mode: filter_mode_str.to_string(),
            included_flair_ids: filter.included_flair_ids.into_iter().flatten().collect(),
            excluded_flair_ids: filter.excluded_flair_ids.into_iter().flatten().collect(),
        })
    }

    /// Clear all flair filters for a board
    async fn clear_flair_filters(&self, ctx: &Context<'_>, board_id: ID) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        let deleted = diesel::delete(
            user_flair_filters::table
                .filter(user_flair_filters::user_id.eq(user.id))
                .filter(user_flair_filters::board_id.eq(board_uuid)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(deleted > 0)
    }
}
