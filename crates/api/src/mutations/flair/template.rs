use crate::{
    LoggedInUser,
    helpers::validation::require_mod_or_admin,
    structs::flair::{
        FlairTemplate, CreateFlairTemplateInput, UpdateFlairTemplateInput
    },
};
use async_graphql::*;
use tinyboards_db::{
    models::{
        user::user::AdminPerms,
        board::board_mods::ModPerms,
    },
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct FlairTemplateMutations;

#[Object]
impl FlairTemplateMutations {
    /// Create a new flair template (admin/mod only)
    /// Site-wide flairs require admin with Flair permission
    /// Board-specific flairs require board mod with Flair permission or admin
    async fn create_flair_template(
        &self,
        ctx: &Context<'_>,
        input: CreateFlairTemplateInput,
    ) -> Result<FlairTemplate> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Validate input
        crate::helpers::flair::validate_flair_template_input(
            &input.text_display,
            input.text_editable.unwrap_or(false),
            &input.emoji_ids,
            input.max_text_length.unwrap_or(5),
        )?;

        // Check permissions based on scope
        if let Some(board_id) = input.board_id {
            // Board-specific flair - check mod permissions
            require_mod_or_admin(
                user,
                pool,
                board_id,
                ModPerms::Flair,
                Some(AdminPerms::Flair),
            )
            .await?;
        } else {
            // Site-wide flair - admin only
            if !user.has_permission(AdminPerms::Flair) {
                return Err(TinyBoardsError::from_message(
                    403,
                    "Site-wide flairs require admin permissions",
                )
                .into());
            }
        }

        // Process style config
        let style_config_json = if let Some(style) = input.style_config {
            Some(style.to_json()?)
        } else {
            None
        };

        // Validate emoji IDs if provided
        if let Some(ref emoji_ids) = input.emoji_ids {
            if emoji_ids.len() > input.max_text_length.unwrap_or(5) as usize {
                return Err(TinyBoardsError::from_message(
                    400,
                    "Too many emojis specified",
                )
                .into());
            }

            // Validate emoji IDs exist and are accessible
            crate::helpers::flair::validate_emoji_ids(pool, emoji_ids).await?;
        }

        // Check flair quota for board-specific flairs
        if let Some(board_id) = input.board_id {
            crate::helpers::flair::check_flair_quota(
                pool,
                board_id,
                input.flair_type.as_str(),
            )
            .await?;
        }

        use tinyboards_db::models::flair::{FlairTemplate as DbFlairTemplate, FlairTemplateForm};
        use tinyboards_db::traits::Crud;

        let form = FlairTemplateForm {
            board_id: input.board_id,
            flair_type: Some(input.flair_type.as_str().to_string()),
            text_display: Some(input.text_display),
            text_color: input.text_color,
            background_color: input.background_color,
            mod_only: Some(input.requires_approval.unwrap_or(false)),
            max_text_length: input.max_text_length,
            is_editable: Some(input.text_editable.unwrap_or(false)),
            display_order: Some(input.display_order.unwrap_or(0)),
            is_active: Some(true),
            ..Default::default()
        };

        let template = DbFlairTemplate::create(pool, &form).await?;

        Ok(crate::structs::flair::FlairTemplate::from(template))
    }

    /// Update an existing flair template (admin/mod only)
    async fn update_flair_template(
        &self,
        ctx: &Context<'_>,
        template_id: i32,
        input: UpdateFlairTemplateInput,
    ) -> Result<FlairTemplate> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        use tinyboards_db::models::flair::{FlairTemplate as DbFlairTemplate, FlairTemplateForm};
        use tinyboards_db::traits::Crud;

        // Get existing template
        let existing_template = DbFlairTemplate::read(pool,template_id).await
            .map_err(|_| TinyBoardsError::from_message(404, "Flair template not found"))?;

        // Check permissions - all flairs are board-specific
        require_mod_or_admin(
            user,
            pool,
            existing_template.board_id,
            ModPerms::Flair,
            Some(AdminPerms::Flair),
        )
        .await?;

        // Validate input if text is being updated
        if let Some(ref text_display) = input.text_display {
            let max_len = input.max_text_length.unwrap_or(existing_template.max_text_length);
            crate::helpers::flair::validate_flair_template_input(
                text_display,
                input.text_editable.unwrap_or(existing_template.is_editable),
                &input.emoji_ids,
                max_len,
            )?;
        }

        let form = FlairTemplateForm {
            text_display: input.text_display,
            is_editable: input.text_editable,
            background_color: input.background_color,
            text_color: input.text_color,
            max_text_length: input.max_text_length,
            display_order: input.display_order,
            mod_only: input.requires_approval,
            is_active: input.is_active,
            ..Default::default()
        };

        let template = DbFlairTemplate::update(pool, template_id, &form).await?;

        Ok(crate::structs::flair::FlairTemplate::from(template))
    }

    /// Delete a flair template (admin/mod only)
    /// This will also remove all assignments of this template
    async fn delete_flair_template(
        &self,
        ctx: &Context<'_>,
        template_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        use tinyboards_db::models::flair::FlairTemplate as DbFlairTemplate;
        use tinyboards_db::traits::Crud;

        // Get existing template
        let existing_template = DbFlairTemplate::read(pool,template_id).await
            .map_err(|_| TinyBoardsError::from_message(404, "Flair template not found"))?;

        // Check permissions - all flairs are board-specific
        require_mod_or_admin(
            user,
            pool,
            existing_template.board_id,
            ModPerms::Flair,
            Some(AdminPerms::Flair),
        )
        .await?;

        // Soft delete the template (sets is_deleted = true)
        DbFlairTemplate::soft_delete(pool, template_id).await?;

        Ok(true)
    }

    /// Reorder flair templates (admin/mod only)
    async fn reorder_flair_templates(
        &self,
        ctx: &Context<'_>,
        board_id: Option<i32>,
        template_ids: Vec<i32>,
    ) -> Result<Vec<FlairTemplate>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        use tinyboards_db::models::flair::FlairTemplate as DbFlairTemplate;

        // Check permissions - board_id is required since all flairs are board-specific
        let board_id = board_id.ok_or_else(|| {
            TinyBoardsError::from_message(400, "board_id is required for reordering")
        })?;

        require_mod_or_admin(
            user,
            pool,
            board_id,
            ModPerms::Flair,
            Some(AdminPerms::Flair),
        )
        .await?;

        // Update display_order for each template
        for (index, template_id) in template_ids.iter().enumerate() {
            DbFlairTemplate::reorder(pool, *template_id, index as i32).await?;
        }

        // Return updated templates
        let templates = DbFlairTemplate::get_by_board(pool, board_id, None).await?;

        Ok(templates.into_iter().map(crate::structs::flair::FlairTemplate::from).collect())
    }
}
