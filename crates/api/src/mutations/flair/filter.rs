use crate::{
    LoggedInUser,
    structs::flair::UpdateFlairFiltersInput,
    queries::flairs::FlairFilters,
};
use async_graphql::*;
use tinyboards_db::{
    models::flair::{
        FlairTemplate,
        user_flair_filter::{UserFlairFilter, UserFlairFilterForm},
    },
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct FlairFilterMutations;

#[Object]
impl FlairFilterMutations {
    /// Update user's flair filter preferences
    /// Users can set which flairs to hide or highlight in their feed
    async fn update_flair_filters(
        &self,
        ctx: &Context<'_>,
        input: UpdateFlairFiltersInput,
    ) -> Result<FlairFilters> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Validate that flair IDs don't overlap
        if let (Some(ref hidden), Some(ref highlighted)) = (&input.hidden_flair_ids, &input.highlighted_flair_ids) {
            let hidden_set: std::collections::HashSet<_> = hidden.iter().collect();
            let highlighted_set: std::collections::HashSet<_> = highlighted.iter().collect();

            if hidden_set.intersection(&highlighted_set).count() > 0 {
                return Err(TinyBoardsError::from_message(
                    400,
                    "A flair cannot be both hidden and highlighted",
                )
                .into());
            }
        }

        // Validate that board_id is provided
        let board_id = input.board_id.ok_or_else(|| {
            TinyBoardsError::from_message(400, "board_id is required for flair filters")
        })?;

        // Validate that all flair IDs exist and are accessible
        let all_flair_ids: Vec<i32> = input
            .hidden_flair_ids
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .chain(input.highlighted_flair_ids.as_ref().unwrap_or(&vec![]).iter())
            .copied()
            .collect();

        if !all_flair_ids.is_empty() {
            for flair_id in &all_flair_ids {
                let template = FlairTemplate::read(pool,*flair_id).await.map_err(|_| {
                    TinyBoardsError::from_message(
                        404,
                        &format!("Flair template with ID {} not found", flair_id),
                    )
                })?;

                // Verify the flair belongs to the specified board
                if template.board_id != board_id {
                    return Err(TinyBoardsError::from_message(
                        400,
                        &format!("Flair template {} does not belong to board {}", flair_id, board_id),
                    )
                    .into());
                }

                // Verify the flair is active
                if !template.is_active {
                    return Err(TinyBoardsError::from_message(
                        400,
                        &format!("Flair template {} is not active", flair_id),
                    )
                    .into());
                }
            }
        }

        // Determine filter mode based on which lists are provided
        let filter_mode = if input.highlighted_flair_ids.is_some() && input.highlighted_flair_ids.as_ref().unwrap().len() > 0 {
            "show" // Show mode: only show posts with these flairs
        } else {
            "hide" // Hide mode: hide posts with these flairs
        };

        // Create or update flair filters
        let form = UserFlairFilterForm {
            user_id: Some(user.id),
            board_id: Some(board_id),
            filter_mode: Some(filter_mode.to_string()),
            excluded_flair_ids: Some(
                input
                    .hidden_flair_ids
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(Some)
                    .collect(),
            ),
            included_flair_ids: Some(
                input
                    .highlighted_flair_ids
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(Some)
                    .collect(),
            ),
        };

        let updated_filter = UserFlairFilter::set_filter(pool, user.id, board_id, &form).await?;

        // Convert to GraphQL return type
        Ok(FlairFilters {
            user_id: updated_filter.user_id,
            board_id: Some(updated_filter.board_id),
            hidden_flair_ids: updated_filter
                .excluded_flair_ids
                .into_iter()
                .filter_map(|id| id)
                .collect(),
            highlighted_flair_ids: updated_filter
                .included_flair_ids
                .into_iter()
                .filter_map(|id| id)
                .collect(),
        })
    }

    /// Clear all flair filters for a user (optionally for a specific board)
    async fn clear_flair_filters(
        &self,
        ctx: &Context<'_>,
        board_id: Option<i32>,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Delete flair filters
        if let Some(board_id) = board_id {
            // Delete filter for specific board
            let deleted_count = UserFlairFilter::remove_filter(pool, user.id, board_id).await?;
            Ok(deleted_count > 0)
        } else {
            // Delete all filters for the user across all boards
            let all_filters = UserFlairFilter::get_for_user(pool, user.id).await.map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to fetch user filters")
            })?;

            let mut deleted_any = false;
            for filter in all_filters {
                let _ = UserFlairFilter::delete(pool, filter.id).await.map_err(|e| {
                    TinyBoardsError::from_error_message(e, 500, "Failed to delete filter")
                })?;
                deleted_any = true;
            }

            Ok(deleted_any)
        }
    }

    /// Add a flair to hidden list
    async fn hide_flair(
        &self,
        ctx: &Context<'_>,
        template_id: i32,
        board_id: Option<i32>,
    ) -> Result<FlairFilters> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Validate board_id is provided
        let board_id = board_id.ok_or_else(|| {
            TinyBoardsError::from_message(400, "board_id is required for flair filters")
        })?;

        // Validate that the flair template exists and belongs to this board
        let template = FlairTemplate::read(pool,template_id).await.map_err(|_| {
            TinyBoardsError::from_message(404, &format!("Flair template with ID {} not found", template_id))
        })?;

        if template.board_id != board_id {
            return Err(TinyBoardsError::from_message(
                400,
                &format!("Flair template {} does not belong to board {}", template_id, board_id),
            )
            .into());
        }

        // Get existing filters or create new
        let existing_filter = UserFlairFilter::get_by_user_and_board(pool, user.id, board_id).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to fetch filters"))?;

        let (mut excluded_ids, mut included_ids) = if let Some(filter) = existing_filter {
            (
                filter.excluded_flair_ids.into_iter().filter_map(|id| id).collect::<Vec<i32>>(),
                filter.included_flair_ids.into_iter().filter_map(|id| id).collect::<Vec<i32>>(),
            )
        } else {
            (vec![], vec![])
        };

        // Add to hidden list if not already there
        if !excluded_ids.contains(&template_id) {
            excluded_ids.push(template_id);
        }

        // Remove from highlighted list if present
        included_ids.retain(|&id| id != template_id);

        // Update filters
        let form = UserFlairFilterForm {
            user_id: Some(user.id),
            board_id: Some(board_id),
            filter_mode: Some("hide".to_string()),
            excluded_flair_ids: Some(excluded_ids.into_iter().map(Some).collect()),
            included_flair_ids: Some(included_ids.into_iter().map(Some).collect()),
        };

        let updated_filter = UserFlairFilter::set_filter(pool, user.id, board_id, &form).await?;

        Ok(FlairFilters {
            user_id: updated_filter.user_id,
            board_id: Some(updated_filter.board_id),
            hidden_flair_ids: updated_filter
                .excluded_flair_ids
                .into_iter()
                .filter_map(|id| id)
                .collect(),
            highlighted_flair_ids: updated_filter
                .included_flair_ids
                .into_iter()
                .filter_map(|id| id)
                .collect(),
        })
    }

    /// Add a flair to highlighted list
    async fn highlight_flair(
        &self,
        ctx: &Context<'_>,
        template_id: i32,
        board_id: Option<i32>,
    ) -> Result<FlairFilters> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Validate board_id is provided
        let board_id = board_id.ok_or_else(|| {
            TinyBoardsError::from_message(400, "board_id is required for flair filters")
        })?;

        // Validate that the flair template exists and belongs to this board
        let template = FlairTemplate::read(pool,template_id).await.map_err(|_| {
            TinyBoardsError::from_message(404, &format!("Flair template with ID {} not found", template_id))
        })?;

        if template.board_id != board_id {
            return Err(TinyBoardsError::from_message(
                400,
                &format!("Flair template {} does not belong to board {}", template_id, board_id),
            )
            .into());
        }

        // Get existing filters or create new
        let existing_filter = UserFlairFilter::get_by_user_and_board(pool, user.id, board_id).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to fetch filters"))?;

        let (mut excluded_ids, mut included_ids) = if let Some(filter) = existing_filter {
            (
                filter.excluded_flair_ids.into_iter().filter_map(|id| id).collect::<Vec<i32>>(),
                filter.included_flair_ids.into_iter().filter_map(|id| id).collect::<Vec<i32>>(),
            )
        } else {
            (vec![], vec![])
        };

        // Add to highlighted list if not already there
        if !included_ids.contains(&template_id) {
            included_ids.push(template_id);
        }

        // Remove from hidden list if present
        excluded_ids.retain(|&id| id != template_id);

        // Update filters with "show" mode
        let form = UserFlairFilterForm {
            user_id: Some(user.id),
            board_id: Some(board_id),
            filter_mode: Some("show".to_string()),
            excluded_flair_ids: Some(excluded_ids.into_iter().map(Some).collect()),
            included_flair_ids: Some(included_ids.into_iter().map(Some).collect()),
        };

        let updated_filter = UserFlairFilter::set_filter(pool, user.id, board_id, &form).await?;

        Ok(FlairFilters {
            user_id: updated_filter.user_id,
            board_id: Some(updated_filter.board_id),
            hidden_flair_ids: updated_filter
                .excluded_flair_ids
                .into_iter()
                .filter_map(|id| id)
                .collect(),
            highlighted_flair_ids: updated_filter
                .included_flair_ids
                .into_iter()
                .filter_map(|id| id)
                .collect(),
        })
    }

    /// Remove a flair from both hidden and highlighted lists
    async fn unhide_flair(
        &self,
        ctx: &Context<'_>,
        template_id: i32,
        board_id: Option<i32>,
    ) -> Result<FlairFilters> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Validate board_id is provided
        let board_id = board_id.ok_or_else(|| {
            TinyBoardsError::from_message(400, "board_id is required for flair filters")
        })?;

        // Get existing filters
        let existing_filter = UserFlairFilter::get_by_user_and_board(pool, user.id, board_id).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to fetch filters"))?
            .ok_or_else(|| TinyBoardsError::from_message(404, "No filters found for this board"))?;

        // Remove from both lists
        let mut excluded_ids: Vec<i32> = existing_filter
            .excluded_flair_ids
            .into_iter()
            .filter_map(|id| id)
            .collect();
        let mut included_ids: Vec<i32> = existing_filter
            .included_flair_ids
            .into_iter()
            .filter_map(|id| id)
            .collect();

        excluded_ids.retain(|&id| id != template_id);
        included_ids.retain(|&id| id != template_id);

        // Update filters
        let form = UserFlairFilterForm {
            user_id: Some(user.id),
            board_id: Some(board_id),
            filter_mode: Some(existing_filter.filter_mode.clone()),
            excluded_flair_ids: Some(excluded_ids.into_iter().map(Some).collect()),
            included_flair_ids: Some(included_ids.into_iter().map(Some).collect()),
        };

        let updated_filter = UserFlairFilter::set_filter(pool, user.id, board_id, &form).await?;

        Ok(FlairFilters {
            user_id: updated_filter.user_id,
            board_id: Some(updated_filter.board_id),
            hidden_flair_ids: updated_filter
                .excluded_flair_ids
                .into_iter()
                .filter_map(|id| id)
                .collect(),
            highlighted_flair_ids: updated_filter
                .included_flair_ids
                .into_iter()
                .filter_map(|id| id)
                .collect(),
        })
    }
}
