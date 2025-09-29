use crate::{
    LoggedInUser,
    structs::emoji::{EmojiObject, ListEmojisInput, EmojiScope},
};
use async_graphql::*;
use tinyboards_db::{
    models::{emoji::emoji::Emoji, user::user::AdminPerms},
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct EmojiQueries;

#[Object]
impl EmojiQueries {
    /// List all available emojis for a given context (site + board emojis)
    async fn list_emojis(
        &self,
        ctx: &Context<'_>,
        input: Option<ListEmojisInput>,
    ) -> Result<Vec<EmojiObject>> {
        let pool = ctx.data::<DbPool>()?;

        let input = input.unwrap_or_default();
        let board_id = input.board_id;

        let emojis = match (&input.search, input.scope) {
            // Handle search queries
            (Some(search_term), Some(EmojiScope::Site)) => {
                Emoji::search_site_emojis(pool, search_term).await?
            }
            (Some(search_term), Some(EmojiScope::Board)) => {
                if let Some(board_id) = board_id {
                    Emoji::search_board_emojis(pool, board_id, search_term).await?
                } else {
                    return Err(Error::new("board_id is required when scope is Board"));
                }
            }
            (Some(search_term), None) => {
                Emoji::search_all_available_emojis(pool, board_id, search_term).await?
            }
            // Handle regular listing without search
            (None, Some(EmojiScope::Site)) => {
                Emoji::list_site_emojis(pool).await?
            }
            (None, Some(EmojiScope::Board)) => {
                if let Some(board_id) = board_id {
                    Emoji::list_board_emojis(pool, board_id).await?
                } else {
                    return Err(Error::new("board_id is required when scope is Board"));
                }
            }
            (None, None) => {
                Emoji::list_all_available_emojis(pool, board_id).await?
            }
        };

        Ok(emojis.into_iter().map(EmojiObject::from).collect())
    }

    /// Get a specific emoji by ID
    async fn get_emoji(&self, ctx: &Context<'_>, emoji_id: i32) -> Result<Option<EmojiObject>> {
        let pool = ctx.data::<DbPool>()?;

        match Emoji::read(pool, emoji_id).await {
            Ok(emoji) => Ok(Some(EmojiObject::from(emoji))),
            Err(_) => Ok(None),
        }
    }

    /// Get all emojis for administrative purposes (includes inactive ones)
    async fn get_all_emojis_admin(
        &self,
        ctx: &Context<'_>,
        board_id: Option<i32>,
    ) -> Result<Vec<EmojiObject>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Check if user has admin permissions
        if !user.has_permission(AdminPerms::Emoji) {
            return Err(TinyBoardsError::from_message(403, "Insufficient permissions").into());
        }

        let emojis = if let Some(board_id) = board_id {
            Emoji::list_all_for_board_admin(pool, board_id).await?
        } else {
            Emoji::list_all_site_admin(pool).await?
        };

        Ok(emojis.into_iter().map(EmojiObject::from).collect())
    }
}

impl Default for ListEmojisInput {
    fn default() -> Self {
        Self {
            board_id: None,
            scope: None,
            active_only: Some(true),
            limit: Some(50),
            offset: Some(0),
            search: None,
        }
    }
}