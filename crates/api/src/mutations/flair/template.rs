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
            input.is_editable.unwrap_or(false),
            &input.emoji_ids,
            input.max_emoji_count.unwrap_or(5),
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
            Some(serde_json::to_value(style).map_err(|e| {
                TinyBoardsError::from_message(400, &format!("Invalid style config: {}", e))
            })?)
        } else {
            None
        };

        // Validate emoji IDs if provided
        if let Some(ref emoji_ids) = input.emoji_ids {
            if emoji_ids.len() > input.max_emoji_count.unwrap_or(5) as usize {
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

        // Generate template_name from text_display (remove special chars, lowercase, limit to 50 chars)
        let template_name = input.text_display
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '_' || *c == '-')
            .collect::<String>()
            .trim()
            .to_lowercase()
            .chars()
            .take(50)
            .collect::<String>();

        let form = FlairTemplateForm {
            board_id: input.board_id,
            flair_type: Some(input.flair_type.as_str().to_string()),
            template_name: Some(template_name),
            text_display: Some(input.text_display),
            text_color: input.text_color,
            background_color: input.background_color,
            style_config: style_config_json,
            emoji_ids: input.emoji_ids.map(|ids| ids.into_iter().map(Some).collect()),
            mod_only: Some(input.requires_approval.unwrap_or(false)),
            max_emoji_count: input.max_emoji_count,
            is_editable: Some(input.is_editable.unwrap_or(false)),
            display_order: Some(input.display_order.unwrap_or(0)),
            is_active: Some(true),
            created_by: Some(user.id),
            category_id: input.category_id.map(Some),
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
            let max_len = input.max_emoji_count.unwrap_or(existing_template.max_emoji_count);
            crate::helpers::flair::validate_flair_template_input(
                text_display,
                input.is_editable.unwrap_or(existing_template.is_editable),
                &input.emoji_ids,
                max_len,
            )?;
        }

        // Process style config if provided
        let style_config_json = if let Some(style) = input.style_config {
            Some(serde_json::to_value(style).map_err(|e| {
                TinyBoardsError::from_message(400, &format!("Invalid style config: {}", e))
            })?)
        } else {
            None
        };

        let form = FlairTemplateForm {
            text_display: input.text_display,
            is_editable: input.is_editable,
            background_color: input.background_color,
            text_color: input.text_color,
            style_config: style_config_json,
            emoji_ids: input.emoji_ids.map(|ids| ids.into_iter().map(Some).collect()),
            max_emoji_count: input.max_emoji_count,
            category_id: input.category_id.map(Some),
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

    /// Duplicate a flair template (admin/mod only)
    async fn duplicate_flair_template(
        &self,
        ctx: &Context<'_>,
        template_id: i32,
    ) -> Result<FlairTemplate> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        use tinyboards_db::models::flair::{FlairTemplate as DbFlairTemplate, FlairTemplateForm};
        use tinyboards_db::traits::Crud;

        // Get existing template
        let existing_template = DbFlairTemplate::read(pool, template_id).await
            .map_err(|_| TinyBoardsError::from_message(404, "Flair template not found"))?;

        // Check permissions
        require_mod_or_admin(
            user,
            pool,
            existing_template.board_id,
            ModPerms::Flair,
            Some(AdminPerms::Flair),
        )
        .await?;

        // Create duplicate with "(Copy)" appended to template name
        let new_template_name = format!("{} (Copy)", existing_template.template_name);
        let new_text_display = format!("{} (Copy)", existing_template.text_display);

        let form = FlairTemplateForm {
            board_id: Some(existing_template.board_id),
            flair_type: Some(existing_template.flair_type),
            template_name: Some(new_template_name),
            text_display: Some(new_text_display),
            text_color: Some(existing_template.text_color),
            background_color: Some(existing_template.background_color),
            style_config: Some(existing_template.style_config),
            emoji_ids: Some(existing_template.emoji_ids),
            mod_only: Some(existing_template.mod_only),
            is_editable: Some(existing_template.is_editable),
            max_emoji_count: Some(existing_template.max_emoji_count),
            requires_approval: Some(existing_template.requires_approval),
            display_order: Some(existing_template.display_order + 1),
            is_active: Some(true),
            created_by: Some(user.id),
            category_id: existing_template.category_id.map(Some),
            ..Default::default()
        };

        let template = DbFlairTemplate::create(pool, &form).await?;

        Ok(crate::structs::flair::FlairTemplate::from(template))
    }

    /// Bulk toggle active status for flair templates (admin/mod only)
    async fn bulk_toggle_flair_templates(
        &self,
        ctx: &Context<'_>,
        template_ids: Vec<i32>,
        is_active: bool,
    ) -> Result<Vec<FlairTemplate>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        use tinyboards_db::models::flair::{FlairTemplate as DbFlairTemplate, FlairTemplateForm};
        use tinyboards_db::traits::Crud;

        let mut results: Vec<FlairTemplate> = Vec::new();

        // Process each template
        for template_id in template_ids {
            // Get template to check permissions
            let template = match DbFlairTemplate::read(pool, template_id).await {
                Ok(t) => t,
                Err(_) => continue, // Skip if not found
            };

            // Check permissions
            if require_mod_or_admin(
                user,
                pool,
                template.board_id,
                ModPerms::Flair,
                Some(AdminPerms::Flair),
            )
            .await
            .is_err()
            {
                continue; // Skip if no permission
            }

            let form = FlairTemplateForm {
                is_active: Some(is_active),
                ..Default::default()
            };

            match DbFlairTemplate::update(pool, template_id, &form).await {
                Ok(updated) => results.push(crate::structs::flair::FlairTemplate::from(updated)),
                Err(_) => continue,
            }
        }

        Ok(results)
    }
}
