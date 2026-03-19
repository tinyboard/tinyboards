use crate::{
    LoggedInUser,
    structs::emoji::{EmojiObject, EmojiScope, ListEmojisInput},
};
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbEmojiScope,
    models::{
        emoji::Emoji as DbEmoji,
        user::user::AdminPerms,
    },
    schema::{emoji, emoji_keywords},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

#[derive(Default)]
pub struct EmojiQueries;

#[Object]
impl EmojiQueries {
    /// List all available emojis for a given context
    async fn list_emojis(
        &self,
        ctx: &Context<'_>,
        input: Option<ListEmojisInput>,
    ) -> Result<Vec<EmojiObject>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let input = input.unwrap_or_default();

        let board_uuid: Option<Uuid> = if let Some(ref bid) = input.board_id {
            Some(bid.parse().map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?)
        } else {
            None
        };

        let limit = input.limit.unwrap_or(50).min(200) as i64;
        let offset = input.offset.unwrap_or(0) as i64;

        let mut query = emoji::table
            .filter(emoji::is_active.eq(true))
            .order(emoji::shortcode.asc())
            .into_boxed();

        // Filter by scope
        match input.scope {
            Some(EmojiScope::Site) => {
                query = query.filter(emoji::scope.eq(DbEmojiScope::Global));
            }
            Some(EmojiScope::Board) => {
                query = query.filter(emoji::scope.eq(DbEmojiScope::Board));
                if let Some(bid) = board_uuid {
                    query = query.filter(emoji::board_id.eq(bid));
                } else {
                    return Err(Error::new("board_id is required when scope is Board"));
                }
            }
            None => {
                // Return all available emojis: site-wide + board-specific
                if let Some(bid) = board_uuid {
                    query = query.filter(
                        emoji::scope.eq(DbEmojiScope::Global)
                            .or(emoji::board_id.eq(bid))
                    );
                }
                // If no board_id, return only global emojis
                else {
                    query = query.filter(emoji::scope.eq(DbEmojiScope::Global));
                }
            }
        }

        // Filter by category
        if let Some(ref category) = input.category {
            query = query.filter(emoji::category.eq(category));
        }

        // Search by shortcode or keywords
        if let Some(ref search) = input.search {
            let search_pattern = format!("%{}%", search);

            // Get emoji IDs matching keywords
            let keyword_emoji_ids: Vec<Uuid> = emoji_keywords::table
                .filter(emoji_keywords::keyword.ilike(search_pattern.clone()))
                .select(emoji_keywords::emoji_id)
                .load(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            query = query.filter(
                emoji::shortcode.ilike(search_pattern)
                    .or(emoji::id.eq_any(keyword_emoji_ids))
            );
        }

        let emojis: Vec<DbEmoji> = query
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(emojis.into_iter().map(EmojiObject::from).collect())
    }

    /// Get a specific emoji by ID
    async fn get_emoji(&self, ctx: &Context<'_>, emoji_id: ID) -> Result<Option<EmojiObject>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let emoji_uuid: Uuid = emoji_id.parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid emoji ID".into()))?;

        let result: Option<DbEmoji> = emoji::table
            .find(emoji_uuid)
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(result.map(EmojiObject::from))
    }

    /// Get all emojis for administrative purposes (includes inactive ones)
    async fn get_all_emojis_admin(
        &self,
        ctx: &Context<'_>,
        board_id: Option<ID>,
    ) -> Result<Vec<EmojiObject>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        if !user.has_permission(AdminPerms::Emoji) {
            return Err(TinyBoardsError::from_message(403, "Insufficient permissions").into());
        }

        let mut query = emoji::table
            .order(emoji::shortcode.asc())
            .into_boxed();

        if let Some(ref bid) = board_id {
            let board_uuid: Uuid = bid.parse()
                .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;
            query = query.filter(emoji::board_id.eq(board_uuid));
        }

        let emojis: Vec<DbEmoji> = query
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(emojis.into_iter().map(EmojiObject::from).collect())
    }
}
