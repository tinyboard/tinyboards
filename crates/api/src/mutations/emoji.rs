use crate::{
    LoggedInUser,
    structs::emoji::{CreateEmojiInput, EmojiObject, UpdateEmojiInput},
    helpers::files::emoji::{upload_emoji_file, delete_emoji_file},
    utils::emoji::reprocess_all_content_with_emojis,
};
use async_graphql::*;
use tinyboards_db::{
    models::{
        emoji::emoji::{Emoji, EmojiForm},
        user::user::AdminPerms,
        board::{boards::Board as DbBoard, board_mods::ModPerms},
    },
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;
use chrono::Utc;

#[derive(Default)]
pub struct EmojiMutations;

#[Object]
impl EmojiMutations {
    /// Create a new emoji (admin/mod only)
    async fn create_emoji(
        &self,
        ctx: &Context<'_>,
        input: CreateEmojiInput,
    ) -> Result<EmojiObject> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Determine scope and validate permissions
        let emoji_scope = input.emoji_scope.unwrap_or_else(|| "site".to_string());

        match emoji_scope.as_str() {
            "site" => {
                if !user.has_permission(AdminPerms::Emoji) {
                    return Err(TinyBoardsError::from_message(403, "Insufficient permissions to create site emojis").into());
                }
            }
            "board" => {
                let board_id = input.board_id.ok_or_else(|| TinyBoardsError::from_message(400, "board_id is required for board emojis"))?;

                // Check if user is admin or board mod with emoji permissions
                let has_permission = if user.has_permission(AdminPerms::Emoji) {
                    true
                } else {
                    let m = DbBoard::board_get_mod(pool, board_id, user.id).await;
                    match m {
                        Ok(m_opt) => match m_opt {
                            Some(m) => m.has_permission(ModPerms::Emoji),
                            None => false,
                        },
                        Err(_) => false,
                    }
                };

                if !has_permission {
                    return Err(TinyBoardsError::from_message(403, "Insufficient permissions to create board emojis").into());
                }
            }
            _ => return Err(TinyBoardsError::from_message(400, "Invalid emoji scope. Must be 'site' or 'board'").into()),
        }

        // Upload the emoji file
        let upload_url = upload_emoji_file(input.image_file, input.shortcode.clone(), user.id, ctx).await?;

        let emoji_form = EmojiForm {
            shortcode: Some(input.shortcode),
            image_url: Some(upload_url.into()),
            alt_text: Some(input.alt_text),
            category: Some(input.category),
            board_id: input.board_id,
            created_by_user_id: Some(user.id),
            is_active: Some(true),
            usage_count: Some(0),
            emoji_scope: Some(emoji_scope),
            updated: Some(Utc::now().naive_utc()),
        };

        let emoji = Emoji::create(pool, &emoji_form).await?;
        Ok(EmojiObject::from(emoji))
    }

    /// Update an existing emoji (admin/mod only)
    async fn update_emoji(
        &self,
        ctx: &Context<'_>,
        emoji_id: i32,
        input: UpdateEmojiInput,
    ) -> Result<EmojiObject> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Get the existing emoji to check permissions
        let existing_emoji = Emoji::read(pool, emoji_id).await
            .map_err(|_| TinyBoardsError::from_message(404, "Emoji not found"))?;

        // Check permissions based on emoji scope
        match existing_emoji.emoji_scope.as_str() {
            "site" => {
                if !user.has_permission(AdminPerms::Emoji) {
                    return Err(TinyBoardsError::from_message(403, "Insufficient permissions to update site emojis").into());
                }
            }
            "board" => {
                let board_id = existing_emoji.board_id.ok_or_else(|| TinyBoardsError::from_message(400, "Board emoji missing board_id"))?;

                let has_permission = if user.has_permission(AdminPerms::Emoji) {
                    true
                } else {
                    let m = DbBoard::board_get_mod(pool, board_id, user.id).await;
                    match m {
                        Ok(m_opt) => match m_opt {
                            Some(m) => m.has_permission(ModPerms::Emoji),
                            None => false,
                        },
                        Err(_) => false,
                    }
                };

                if !has_permission {
                    return Err(TinyBoardsError::from_message(403, "Insufficient permissions to update board emojis").into());
                }
            }
            _ => return Err(TinyBoardsError::from_message(400, "Invalid emoji scope").into()),
        }

        // Handle file upload if new image provided
        let new_image_url = if let Some(image_file) = input.image_file {
            let shortcode = input.shortcode.as_ref()
                .unwrap_or(&existing_emoji.shortcode);
            Some(upload_emoji_file(image_file, shortcode.clone(), user.id, ctx).await?.into())
        } else {
            None
        };

        let emoji_form = EmojiForm {
            shortcode: input.shortcode,
            image_url: new_image_url,
            alt_text: input.alt_text,
            category: input.category,
            is_active: input.is_active,
            updated: Some(Utc::now().naive_utc()),
            ..Default::default()
        };

        let emoji = Emoji::update(pool, emoji_id, &emoji_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update emoji"))?;
        Ok(EmojiObject::from(emoji))
    }

    /// Delete an emoji (admin/mod only)
    async fn delete_emoji(&self, ctx: &Context<'_>, emoji_id: i32) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Get the existing emoji to check permissions
        let existing_emoji = Emoji::read(pool, emoji_id).await
            .map_err(|_| TinyBoardsError::from_message(404, "Emoji not found"))?;

        // Check permissions based on emoji scope
        match existing_emoji.emoji_scope.as_str() {
            "site" => {
                if !user.has_permission(AdminPerms::Emoji) {
                    return Err(TinyBoardsError::from_message(403, "Insufficient permissions to delete site emojis").into());
                }
            }
            "board" => {
                let board_id = existing_emoji.board_id.ok_or_else(|| TinyBoardsError::from_message(400, "Board emoji missing board_id"))?;

                let has_permission = if user.has_permission(AdminPerms::Emoji) {
                    true
                } else {
                    let m = DbBoard::board_get_mod(pool, board_id, user.id).await;
                    match m {
                        Ok(m_opt) => match m_opt {
                            Some(m) => m.has_permission(ModPerms::Emoji),
                            None => false,
                        },
                        Err(_) => false,
                    }
                };

                if !has_permission {
                    return Err(TinyBoardsError::from_message(403, "Insufficient permissions to delete board emojis").into());
                }
            }
            _ => return Err(TinyBoardsError::from_message(400, "Invalid emoji scope").into()),
        }

        // Delete the emoji file before deleting the database record
        if let Err(e) = delete_emoji_file(pool, &existing_emoji).await {
            eprintln!("Warning: Failed to delete emoji file: {}", e);
            // Continue with database deletion anyway
        }

        let deleted_count = Emoji::delete(pool, emoji_id).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to delete emoji"))?;
        Ok(deleted_count > 0)
    }

    /// Increment emoji usage count (public)
    async fn use_emoji(&self, ctx: &Context<'_>, emoji_id: i32) -> Result<EmojiObject> {
        let pool = ctx.data::<DbPool>()?;

        let emoji = Emoji::increment_usage(pool, emoji_id).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update emoji usage"))?;
        Ok(EmojiObject::from(emoji))
    }

    /// Batch reprocess existing content with emoji rendering (admin only)
    async fn reprocess_content_emojis(&self, ctx: &Context<'_>, board_id: Option<i32>) -> Result<String> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Check admin permissions
        if !user.has_permission(AdminPerms::Emoji) {
            return Err(TinyBoardsError::from_message(403, "Insufficient permissions").into());
        }

        // Spawn background task for reprocessing
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            if let Err(e) = reprocess_all_content_with_emojis(&pool_clone, board_id).await {
                eprintln!("Error reprocessing content with emojis: {}", e);
            }
        });

        let scope = if board_id.is_some() {
            "board-specific"
        } else {
            "site-wide"
        };

        Ok(format!("Emoji reprocessing task started for {} content", scope))
    }
}