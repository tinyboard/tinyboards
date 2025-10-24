use async_graphql::*;
use tinyboards_db::{
    models::flair::{CreateFlairCategory, FlairCategory as DbFlairCategory, UpdateFlairCategory},
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

use crate::{
    structs::flair::{
        CreateFlairCategoryInput, FlairCategory, ReorderFlairCategoriesInput,
        UpdateFlairCategoryInput,
    },
    LoggedInUser,
};

#[derive(Default)]
pub struct MutationFlairCategories;

#[Object]
impl MutationFlairCategories {
    /// Create a new flair category
    pub async fn create_flair_category(
        &self,
        ctx: &Context<'_>,
        input: CreateFlairCategoryInput,
    ) -> Result<FlairCategory> {
        let pool = ctx.data::<DbPool>()?;
        let v = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Create the category
        let form = CreateFlairCategory {
            board_id: input.board_id,
            name: input.name,
            description: input.description,
            color: input.color,
            display_order: input.display_order,
            created_by: v.id,
        };

        let category = DbFlairCategory::create(pool, form)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to create flair category.")
            })?;

        Ok(FlairCategory::from(category))
    }

    /// Update a flair category
    pub async fn update_flair_category(
        &self,
        ctx: &Context<'_>,
        input: UpdateFlairCategoryInput,
    ) -> Result<FlairCategory> {
        let pool = ctx.data::<DbPool>()?;
        let _v = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        let form = UpdateFlairCategory {
            name: input.name,
            description: input.description,
            color: input.color,
            display_order: input.display_order,
        };

        let category = DbFlairCategory::update(pool, input.id, form)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to update flair category.")
            })?;

        Ok(FlairCategory::from(category))
    }

    /// Delete a flair category
    pub async fn delete_flair_category(
        &self,
        ctx: &Context<'_>,
        category_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let _v = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        DbFlairCategory::delete(pool, category_id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to delete flair category.")
            })?;

        Ok(true)
    }

    /// Reorder flair categories
    pub async fn reorder_flair_categories(
        &self,
        ctx: &Context<'_>,
        order: Vec<ReorderFlairCategoriesInput>,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let _v = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        let updates: Vec<(i32, i32)> = order
            .into_iter()
            .map(|item| (item.id, item.display_order))
            .collect();

        DbFlairCategory::reorder(pool, updates)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(
                    e,
                    500,
                    "Failed to reorder flair categories.",
                )
            })?;

        Ok(true)
    }
}
