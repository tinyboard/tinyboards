use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        flair::{FlairCategory as DbFlairCategory, FlairCategoryInsertForm, FlairCategoryUpdateForm},
    },
    schema::{board_moderators, flair_categories},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{
    structs::flair::{CreateFlairCategoryInput, FlairCategory, UpdateFlairCategoryInput},
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
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let board_uuid: Uuid = input
            .board_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        // Check mod or admin permissions
        if !user.is_admin {
            let _mod: BoardModerator = board_moderators::table
                .filter(board_moderators::board_id.eq(board_uuid))
                .filter(board_moderators::user_id.eq(user.id))
                .filter(board_moderators::is_invite_accepted.eq(true))
                .first(conn)
                .await
                .map_err(|_| {
                    TinyBoardsError::from_message(403, "You must be a moderator to manage flair categories")
                })?;
        }

        let form = FlairCategoryInsertForm {
            board_id: board_uuid,
            name: input.name,
            description: input.description,
            color: input.color,
            display_order: input.display_order.unwrap_or(0),
            created_by: user.id,
        };

        let category: DbFlairCategory = diesel::insert_into(flair_categories::table)
            .values(&form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(FlairCategory::from(category))
    }

    /// Update a flair category
    pub async fn update_flair_category(
        &self,
        ctx: &Context<'_>,
        category_id: ID,
        input: UpdateFlairCategoryInput,
    ) -> Result<FlairCategory> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let category_uuid: Uuid = category_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid category ID".into()))?;

        // Get existing to check board
        let existing: DbFlairCategory = flair_categories::table
            .find(category_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Flair category not found".into()))?;

        if !user.is_admin {
            let _mod: BoardModerator = board_moderators::table
                .filter(board_moderators::board_id.eq(existing.board_id))
                .filter(board_moderators::user_id.eq(user.id))
                .filter(board_moderators::is_invite_accepted.eq(true))
                .first(conn)
                .await
                .map_err(|_| TinyBoardsError::from_message(403, "Insufficient permissions"))?;
        }

        let form = FlairCategoryUpdateForm {
            name: input.name,
            description: input.description.map(Some),
            color: input.color.map(Some),
            display_order: input.display_order,
        };

        let category: DbFlairCategory = diesel::update(flair_categories::table.find(category_uuid))
            .set(&form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(FlairCategory::from(category))
    }

    /// Delete a flair category
    pub async fn delete_flair_category(&self, ctx: &Context<'_>, category_id: ID) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let category_uuid: Uuid = category_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid category ID".into()))?;

        let existing: DbFlairCategory = flair_categories::table
            .find(category_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Flair category not found".into()))?;

        if !user.is_admin {
            let _mod: BoardModerator = board_moderators::table
                .filter(board_moderators::board_id.eq(existing.board_id))
                .filter(board_moderators::user_id.eq(user.id))
                .filter(board_moderators::is_invite_accepted.eq(true))
                .first(conn)
                .await
                .map_err(|_| TinyBoardsError::from_message(403, "Insufficient permissions"))?;
        }

        let deleted = diesel::delete(flair_categories::table.find(category_uuid))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(deleted > 0)
    }
}
