use crate::{
    LoggedInUser,
    helpers::validation::require_mod_or_admin,
    structs::flair::{
        PostFlair, UserFlair, AssignPostFlairInput, AssignUserFlairInput, ApprovalStatus,
    },
};
use async_graphql::*;
use tinyboards_db::{
    models::{
        user::user::AdminPerms,
        board::board_mods::ModPerms,
        post::posts::Post,
        flair::{
            post_flair::{PostFlair as PostFlairDb, PostFlairForm},
            user_flair::{UserFlair as UserFlairDb, UserFlairForm},
            flair_template::FlairTemplate as FlairTemplateDb,
        },
    },
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct FlairAssignmentMutations;

#[Object]
impl FlairAssignmentMutations {
    /// Assign flair to a post
    /// Post author or moderators can assign post flairs
    async fn assign_post_flair(
        &self,
        ctx: &Context<'_>,
        input: AssignPostFlairInput,
    ) -> Result<PostFlair> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Get post to verify it exists and check permissions
        let post = Post::read(pool, input.post_id).await
            .map_err(|_| TinyBoardsError::from_message(404, "Post not found"))?;

        // Check permissions - post author or mod/admin
        let is_author = post.creator_id == user.id;
        let is_mod_or_admin = if is_author {
            true
        } else {
            match require_mod_or_admin(
                user,
                pool,
                post.board_id,
                ModPerms::Flair,
                Some(AdminPerms::Flair),
            )
            .await
            {
                Ok(_) => true,
                Err(_) => false,
            }
        };

        if !is_mod_or_admin {
            return Err(TinyBoardsError::from_message(
                403,
                "Only post author or moderators can assign post flairs",
            )
            .into());
        }

        // Get template if specified and validate
        let mut background_color: Option<String> = None;
        let mut text_color: Option<String> = None;

        if let Some(template_id) = input.template_id {
            let template = FlairTemplateDb::read(pool,template_id).await
                .map_err(|_| TinyBoardsError::from_message(404, "Flair template not found"))?;

            // Validate template is for posts and is active
            if template.flair_type != "post" {
                return Err(TinyBoardsError::from_message(400, "Template is not for posts").into());
            }
            if !template.is_active {
                return Err(TinyBoardsError::from_message(400, "Template is not active").into());
            }

            // Validate template belongs to the same board as the post
            if template.board_id != post.board_id {
                return Err(TinyBoardsError::from_message(400, "Template is not for this board").into());
            }

            // Validate text if template is not editable
            if !template.is_editable && input.text_display != template.text_display {
                return Err(TinyBoardsError::from_message(
                    400,
                    "This template does not allow text editing",
                )
                .into());
            }

            // Validate text length (max 150 characters)
            if input.text_display.len() > 150 {
                return Err(TinyBoardsError::from_message(
                    400,
                    "Flair text exceeds maximum length of 150 characters",
                )
                .into());
            }

            // Copy template styles
            background_color = Some(template.background_color.clone());
            text_color = Some(template.text_color.clone());
        }

        // Validate emoji IDs if provided
        if let Some(ref emoji_ids) = input.emoji_ids {
            crate::helpers::flair::validate_emoji_ids(pool, emoji_ids).await?;
        }

        // Create or update post flair assignment
        let template_id = input.template_id.ok_or_else(|| {
            TinyBoardsError::from_message(400, "Template ID is required")
        })?;

        let form = PostFlairForm {
            post_id: Some(input.post_id),
            flair_template_id: Some(template_id),
            custom_text: Some(Some(input.text_display)),
            custom_text_color: Some(text_color),
            custom_background_color: Some(background_color),
            assigned_by: Some(user.id),
            is_original_author: Some(is_author),
        };

        let flair = PostFlairDb::assign_to_post(pool, input.post_id, &form).await?;

        Ok(flair.into())
    }

    /// Remove flair from a post
    /// Post author or moderators can remove post flairs
    async fn remove_post_flair(
        &self,
        ctx: &Context<'_>,
        post_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Get post to verify it exists and check permissions
        let post = Post::read(pool, post_id).await
            .map_err(|_| TinyBoardsError::from_message(404, "Post not found"))?;

        // Check permissions - post author or mod/admin
        let is_author = post.creator_id == user.id;
        if !is_author {
            require_mod_or_admin(
                user,
                pool,
                post.board_id,
                ModPerms::Flair,
                Some(AdminPerms::Flair),
            )
            .await?;
        }

        // Delete post flair
        let deleted_count = PostFlairDb::remove_from_post(pool, post_id).await?;

        Ok(deleted_count > 0)
    }

    /// Assign flair to a user
    /// User can assign to themselves, moderators can assign to others
    async fn assign_user_flair(
        &self,
        ctx: &Context<'_>,
        input: AssignUserFlairInput,
    ) -> Result<UserFlair> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        let is_self = input.user_id == user.id;
        let mut requires_approval = false;

        // Check permissions
        if !is_self {
            // Assigning to someone else - requires mod/admin
            if let Some(board_id) = input.board_id {
                require_mod_or_admin(
                    user,
                    pool,
                    board_id,
                    ModPerms::Flair,
                    Some(AdminPerms::Flair),
                )
                .await?;
            } else {
                // Site-wide user flair - admin only
                if !user.has_permission(AdminPerms::Flair) {
                    return Err(TinyBoardsError::from_message(
                        403,
                        "Site-wide user flairs require admin permissions",
                    )
                    .into());
                }
            }
        }

        // Get template if specified and validate
        let mut background_color: Option<String> = None;
        let mut text_color: Option<String> = None;

        if let Some(template_id) = input.template_id {
            let template = FlairTemplateDb::read(pool,template_id).await
                .map_err(|_| TinyBoardsError::from_message(404, "Flair template not found"))?;

            // Validate template is for users and is active
            if template.flair_type != "user" {
                return Err(TinyBoardsError::from_message(400, "Template is not for users").into());
            }
            if !template.is_active {
                return Err(TinyBoardsError::from_message(400, "Template is not active").into());
            }

            // Validate template belongs to the same board if board_id is specified
            if let Some(board_id) = input.board_id {
                if template.board_id != board_id {
                    return Err(TinyBoardsError::from_message(400, "Template is not for this board").into());
                }
            }

            // Check if approval is required
            if is_self && template.requires_approval {
                requires_approval = true;
            }

            // Validate text if template is not editable
            if !template.is_editable && input.text_display != template.text_display {
                return Err(TinyBoardsError::from_message(
                    400,
                    "This template does not allow text editing",
                )
                .into());
            }

            // Validate text length (max 150 characters)
            if input.text_display.len() > 150 {
                return Err(TinyBoardsError::from_message(
                    400,
                    "Flair text exceeds maximum length of 150 characters",
                )
                .into());
            }

            // Copy template styles
            background_color = Some(template.background_color.clone());
            text_color = Some(template.text_color.clone());
        }

        // Validate emoji IDs if provided
        if let Some(ref emoji_ids) = input.emoji_ids {
            crate::helpers::flair::validate_emoji_ids(pool, emoji_ids).await?;
        }

        // Determine if auto-approval should be granted
        let auto_approve = if is_self && requires_approval {
            false // User is assigning to themselves and template requires approval
        } else {
            true // Either moderator is assigning, or template doesn't require approval
        };

        // Create or update user flair assignment
        let template_id = input.template_id.ok_or_else(|| {
            TinyBoardsError::from_message(400, "Template ID is required")
        })?;

        let board_id = input.board_id.ok_or_else(|| {
            TinyBoardsError::from_message(400, "Board ID is required")
        })?;

        let form = UserFlairForm {
            user_id: Some(input.user_id),
            board_id: Some(board_id),
            flair_template_id: Some(template_id),
            custom_text: Some(Some(input.text_display)),
            custom_text_color: Some(text_color),
            custom_background_color: Some(background_color),
            is_approved: Some(auto_approve),
            approved_by: if !is_self && auto_approve { Some(Some(user.id)) } else { Some(None) },
            is_self_assigned: Some(is_self),
        };

        let flair = UserFlairDb::assign_to_user(pool, input.user_id, board_id, &form, auto_approve).await?;

        Ok(flair.into())
    }

    /// Remove flair from a user
    /// User can remove their own, moderators can remove others
    async fn remove_user_flair(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
        board_id: Option<i32>,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        let is_self = user_id == user.id;

        // Check permissions
        if !is_self {
            if let Some(board_id) = board_id {
                require_mod_or_admin(
                    user,
                    pool,
                    board_id,
                    ModPerms::Flair,
                    Some(AdminPerms::Flair),
                )
                .await?;
            } else {
                if !user.has_permission(AdminPerms::Flair) {
                    return Err(TinyBoardsError::from_message(
                        403,
                        "Cannot remove other users' site-wide flairs",
                    )
                    .into());
                }
            }
        }

        // Delete user flair
        let board_id = board_id.ok_or_else(|| {
            TinyBoardsError::from_message(400, "Board ID is required")
        })?;

        let deleted_count = UserFlairDb::remove_from_user(pool, user_id, board_id).await?;

        Ok(deleted_count > 0)
    }

    /// Approve a pending user flair (moderator only)
    async fn approve_user_flair(
        &self,
        ctx: &Context<'_>,
        flair_id: i32,
        approved: bool,
    ) -> Result<UserFlair> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Get flair to verify it exists and check permissions
        let flair = UserFlairDb::read(pool, flair_id).await
            .map_err(|_| TinyBoardsError::from_message(404, "User flair not found"))?;

        // Check permissions - must be moderator or admin for the board
        require_mod_or_admin(
            user,
            pool,
            flair.board_id,
            ModPerms::Flair,
            Some(AdminPerms::Flair),
        )
        .await?;

        // Update approval status
        let updated_flair = if approved {
            UserFlairDb::approve(pool, flair_id, user.id).await?
        } else {
            UserFlairDb::reject(pool, flair_id, user.id).await?
        };

        Ok(updated_flair.into())
    }

    /// Bulk approve or reject pending user flairs (moderator only)
    async fn bulk_moderate_user_flairs(
        &self,
        ctx: &Context<'_>,
        flair_ids: Vec<i32>,
        approved: bool,
    ) -> Result<Vec<UserFlair>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        let mut results: Vec<UserFlair> = Vec::new();

        // Process each flair ID
        for flair_id in flair_ids {
            // Get flair to verify it exists and check permissions
            let flair = match UserFlairDb::read(pool, flair_id).await {
                Ok(f) => f,
                Err(_) => {
                    // Skip flairs that don't exist
                    continue;
                }
            };

            // Check permissions - must be moderator or admin for the board
            if require_mod_or_admin(
                user,
                pool,
                flair.board_id,
                ModPerms::Flair,
                Some(AdminPerms::Flair),
            )
            .await
            .is_err()
            {
                // Skip flairs where user doesn't have permission
                continue;
            }

            // Update approval status
            let updated_flair = if approved {
                match UserFlairDb::approve(pool, flair_id, user.id).await {
                    Ok(f) => f,
                    Err(_) => continue,
                }
            } else {
                match UserFlairDb::reject(pool, flair_id, user.id).await {
                    Ok(f) => f,
                    Err(_) => continue,
                }
            };

            results.push(updated_flair.into());
        }

        Ok(results)
    }

    /// Update post flairs - remove all existing and assign new ones (moderator only)
    async fn update_post_flairs(
        &self,
        ctx: &Context<'_>,
        post_id: i32,
        flair_ids: Vec<i32>,
    ) -> Result<Vec<PostFlair>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Get post to verify it exists and check permissions
        let post = Post::read(pool, post_id).await
            .map_err(|_| TinyBoardsError::from_message(404, "Post not found"))?;

        // Check permissions - mod/admin only
        require_mod_or_admin(
            user,
            pool,
            post.board_id,
            ModPerms::Flair,
            Some(AdminPerms::Flair),
        )
        .await?;

        // Remove all existing post flairs
        PostFlairDb::remove_from_post(pool, post_id).await?;

        let mut results: Vec<PostFlair> = Vec::new();

        // Assign new flairs
        for template_id in flair_ids {
            // Get template to validate and get default text
            let template = FlairTemplateDb::read(pool, template_id).await
                .map_err(|_| TinyBoardsError::from_message(404, "Flair template not found"))?;

            // Validate template is for posts and is active
            if template.flair_type != "post" {
                continue; // Skip non-post templates
            }
            if !template.is_active {
                continue; // Skip inactive templates
            }

            // Validate template belongs to the same board
            if template.board_id != post.board_id {
                continue; // Skip templates from other boards
            }

            let form = PostFlairForm {
                post_id: Some(post_id),
                flair_template_id: Some(template_id),
                custom_text: Some(Some(template.text_display.clone())),
                custom_text_color: Some(Some(template.text_color.clone())),
                custom_background_color: Some(Some(template.background_color.clone())),
                assigned_by: Some(user.id),
                is_original_author: Some(false),
            };

            match PostFlairDb::assign_to_post(pool, post_id, &form).await {
                Ok(flair) => results.push(flair.into()),
                Err(_) => continue,
            }
        }

        Ok(results)
    }
}
