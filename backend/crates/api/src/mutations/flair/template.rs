use crate::{
    structs::flair::{CreateFlairTemplateInput, FlairTemplate, FlairType, UpdateFlairTemplateInput},
    LoggedInUser,
};
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbFlairType,
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        flair::{FlairTemplate as DbFlairTemplate, FlairTemplateInsertForm, FlairTemplateUpdateForm},
    },
    schema::{board_moderators, flair_templates},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

/// Check if user is a board mod with Content permission or an admin
async fn require_board_mod_or_admin(
    conn: &mut diesel_async::AsyncPgConnection,
    user: &tinyboards_db::models::user::user::User,
    board_uuid: Uuid,
) -> Result<()> {
    if user.is_admin {
        return Ok(());
    }

    let moderator: BoardModerator = board_moderators::table
        .filter(board_moderators::board_id.eq(board_uuid))
        .filter(board_moderators::user_id.eq(user.id))
        .filter(board_moderators::is_invite_accepted.eq(true))
        .first(conn)
        .await
        .map_err(|_| TinyBoardsError::from_message(403, "You are not a moderator of this board"))?;

    if !moderator.has_permission(ModPerms::Content) {
        return Err(TinyBoardsError::from_message(403, "Insufficient permissions to manage flairs").into());
    }
    Ok(())
}

#[derive(Default)]
pub struct FlairTemplateMutations;

#[Object]
impl FlairTemplateMutations {
    /// Create a new flair template (mod/admin only)
    async fn create_flair_template(
        &self,
        ctx: &Context<'_>,
        input: CreateFlairTemplateInput,
    ) -> Result<FlairTemplate> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let board_uuid: Uuid = input
            .board_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        require_board_mod_or_admin(conn, user, board_uuid).await?;

        let db_flair_type = match input.flair_type {
            FlairType::Post => DbFlairType::Post,
            FlairType::User => DbFlairType::User,
        };

        let style_config = if let Some(style) = input.style_config {
            style
                .to_json_value()
                .map_err(|e| TinyBoardsError::from_message(400, &format!("Invalid style: {:?}", e)))?
        } else {
            serde_json::json!({})
        };

        let category_uuid: Option<Uuid> = if let Some(ref cid) = input.category_id {
            Some(
                cid.parse()
                    .map_err(|_| TinyBoardsError::NotFound("Invalid category ID".into()))?,
            )
        } else {
            None
        };

        let form = FlairTemplateInsertForm {
            board_id: board_uuid,
            flair_type: db_flair_type,
            template_name: input.template_name,
            template_key: None,
            text_display: input.text_display,
            text_color: input.text_color.unwrap_or_else(|| "#000000".to_string()),
            background_color: input.background_color.unwrap_or_else(|| "#e0e0e0".to_string()),
            style_config,
            emoji_ids: input
                .emoji_ids
                .map(|ids| ids.into_iter().map(Some).collect())
                .unwrap_or_default(),
            is_mod_only: input.is_mod_only.unwrap_or(false),
            is_editable: input.is_editable.unwrap_or(false),
            max_emoji_count: input.max_emoji_count.unwrap_or(5),
            max_text_length: input.max_text_length.unwrap_or(64),
            is_requires_approval: input.is_requires_approval.unwrap_or(false),
            display_order: input.display_order.unwrap_or(0),
            is_active: true,
            category_id: category_uuid,
            created_by: user.id,
        };

        let template: DbFlairTemplate = diesel::insert_into(flair_templates::table)
            .values(&form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(FlairTemplate::from(template))
    }

    /// Update an existing flair template (mod/admin only)
    async fn update_flair_template(
        &self,
        ctx: &Context<'_>,
        template_id: ID,
        input: UpdateFlairTemplateInput,
    ) -> Result<FlairTemplate> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let template_uuid: Uuid = template_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid template ID".into()))?;

        let existing: DbFlairTemplate = flair_templates::table
            .find(template_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Flair template not found".into()))?;

        require_board_mod_or_admin(conn, user, existing.board_id).await?;

        let style_config = if let Some(style) = input.style_config {
            Some(
                style
                    .to_json_value()
                    .map_err(|e| TinyBoardsError::from_message(400, &format!("Invalid style: {:?}", e)))?,
            )
        } else {
            None
        };

        let category_uuid: Option<Option<Uuid>> = if let Some(ref cid) = input.category_id {
            Some(Some(
                cid.parse()
                    .map_err(|_| TinyBoardsError::NotFound("Invalid category ID".into()))?,
            ))
        } else {
            None
        };

        let form = FlairTemplateUpdateForm {
            flair_type: None,
            template_name: input.template_name,
            template_key: None,
            text_display: input.text_display,
            text_color: input.text_color,
            background_color: input.background_color,
            style_config,
            emoji_ids: input.emoji_ids.map(|ids| ids.into_iter().map(Some).collect()),
            is_mod_only: input.is_mod_only,
            is_editable: input.is_editable,
            max_emoji_count: input.max_emoji_count,
            max_text_length: input.max_text_length,
            is_requires_approval: input.is_requires_approval,
            display_order: input.display_order,
            is_active: input.is_active,
            category_id: category_uuid,
        };

        let updated: DbFlairTemplate = diesel::update(flair_templates::table.find(template_uuid))
            .set(&form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(FlairTemplate::from(updated))
    }

    /// Delete a flair template (deactivate, mod/admin only)
    async fn delete_flair_template(&self, ctx: &Context<'_>, template_id: ID) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let template_uuid: Uuid = template_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid template ID".into()))?;

        let existing: DbFlairTemplate = flair_templates::table
            .find(template_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Flair template not found".into()))?;

        require_board_mod_or_admin(conn, user, existing.board_id).await?;

        // Deactivate rather than hard-delete to preserve existing assignments
        let deleted = diesel::update(flair_templates::table.find(template_uuid))
            .set(flair_templates::is_active.eq(false))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(deleted > 0)
    }
}
