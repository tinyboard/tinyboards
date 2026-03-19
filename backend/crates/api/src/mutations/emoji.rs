use crate::{
    LoggedInUser,
    helpers::files::emoji::upload_emoji_file,
    structs::emoji::{CreateEmojiInput, EmojiObject, EmojiScope, UpdateEmojiInput},
};
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbEmojiScope,
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        emoji::{Emoji as DbEmoji, EmojiInsertForm, EmojiKeywordInsertForm, EmojiUpdateForm},
        user::user::AdminPerms,
    },
    schema::{board_moderators, emoji, emoji_keywords},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

#[derive(Default)]
pub struct EmojiMutations;

/// Check if the user has permission to manage emojis for the given scope
async fn check_emoji_permission(
    conn: &mut diesel_async::AsyncPgConnection,
    user: &tinyboards_db::models::user::user::User,
    scope: &EmojiScope,
    board_id: Option<Uuid>,
) -> Result<()> {
    match scope {
        EmojiScope::Site => {
            if !user.has_permission(AdminPerms::Emoji) {
                return Err(
                    TinyBoardsError::from_message(403, "Insufficient permissions to manage site emojis").into(),
                );
            }
        }
        EmojiScope::Board => {
            let bid = board_id.ok_or_else(|| {
                TinyBoardsError::from_message(400, "board_id is required for board emojis")
            })?;

            if !user.has_permission(AdminPerms::Emoji) {
                let moderator: BoardModerator = board_moderators::table
                    .filter(board_moderators::board_id.eq(bid))
                    .filter(board_moderators::user_id.eq(user.id))
                    .filter(board_moderators::is_invite_accepted.eq(true))
                    .first(conn)
                    .await
                    .map_err(|_| {
                        TinyBoardsError::from_message(403, "You are not a moderator of this board")
                    })?;

                if !moderator.has_permission(ModPerms::Content) {
                    return Err(TinyBoardsError::from_message(
                        403,
                        "Insufficient moderation permissions for emoji management",
                    )
                    .into());
                }
            }
        }
    }
    Ok(())
}

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
        let conn = &mut get_conn(pool).await?;

        let scope = input.scope.unwrap_or(EmojiScope::Site);

        let board_uuid: Option<Uuid> = if let Some(ref bid) = input.board_id {
            Some(
                bid.parse()
                    .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?,
            )
        } else {
            None
        };

        check_emoji_permission(conn, user, &scope, board_uuid).await?;

        let db_scope = match scope {
            EmojiScope::Site => DbEmojiScope::Global,
            EmojiScope::Board => DbEmojiScope::Board,
        };

        // Check for duplicate shortcode
        let existing_count: i64 = emoji::table
            .filter(emoji::shortcode.eq(&input.shortcode))
            .filter(emoji::scope.eq(&db_scope))
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if existing_count > 0 {
            return Err(TinyBoardsError::from_message(
                409,
                "An emoji with this shortcode already exists in this scope",
            )
            .into());
        }

        let insert_form = EmojiInsertForm {
            shortcode: input.shortcode,
            image_url: input.image_url,
            alt_text: input.alt_text,
            category: input.category,
            scope: db_scope,
            board_id: board_uuid,
            created_by: user.id,
            is_active: true,
        };

        let new_emoji: DbEmoji = diesel::insert_into(emoji::table)
            .values(&insert_form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Insert keywords if provided
        if let Some(keywords) = input.keywords {
            let keyword_forms: Vec<EmojiKeywordInsertForm> = keywords
                .into_iter()
                .map(|kw| EmojiKeywordInsertForm {
                    emoji_id: new_emoji.id,
                    keyword: kw,
                })
                .collect();

            if !keyword_forms.is_empty() {
                diesel::insert_into(emoji_keywords::table)
                    .values(&keyword_forms)
                    .execute(conn)
                    .await
                    .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
            }
        }

        Ok(EmojiObject::from(new_emoji))
    }

    /// Update an existing emoji (admin/mod only)
    async fn update_emoji(
        &self,
        ctx: &Context<'_>,
        emoji_id: ID,
        input: UpdateEmojiInput,
    ) -> Result<EmojiObject> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let emoji_uuid: Uuid = emoji_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid emoji ID".into()))?;

        let existing: DbEmoji = emoji::table
            .find(emoji_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Emoji not found".into()))?;

        // Determine scope for permission check
        let scope = match existing.scope {
            DbEmojiScope::Global => EmojiScope::Site,
            DbEmojiScope::Board => EmojiScope::Board,
        };

        check_emoji_permission(conn, user, &scope, existing.board_id).await?;

        let update_form = EmojiUpdateForm {
            shortcode: input.shortcode,
            image_url: input.image_url,
            alt_text: input.alt_text,
            category: input.category,
            scope: None,     // Don't change scope on update
            board_id: None,  // Don't change board_id on update
            is_active: input.is_active,
        };

        let updated: DbEmoji = diesel::update(emoji::table.find(emoji_uuid))
            .set(&update_form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Update keywords if provided
        if let Some(keywords) = input.keywords {
            // Delete existing keywords
            diesel::delete(emoji_keywords::table.filter(emoji_keywords::emoji_id.eq(emoji_uuid)))
                .execute(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            // Insert new keywords
            let keyword_forms: Vec<EmojiKeywordInsertForm> = keywords
                .into_iter()
                .map(|kw| EmojiKeywordInsertForm {
                    emoji_id: emoji_uuid,
                    keyword: kw,
                })
                .collect();

            if !keyword_forms.is_empty() {
                diesel::insert_into(emoji_keywords::table)
                    .values(&keyword_forms)
                    .execute(conn)
                    .await
                    .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
            }
        }

        Ok(EmojiObject::from(updated))
    }

    /// Upload an emoji image file and create the emoji in one step
    async fn upload_emoji(
        &self,
        ctx: &Context<'_>,
        shortcode: String,
        file: Upload,
        category: Option<String>,
        board_id: Option<ID>,
        scope: Option<EmojiScope>,
    ) -> Result<EmojiObject> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let resolved_scope = scope.unwrap_or(EmojiScope::Site);

        let board_uuid: Option<Uuid> = if let Some(ref bid) = board_id {
            Some(
                bid.parse()
                    .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?,
            )
        } else {
            None
        };

        check_emoji_permission(conn, user, &resolved_scope, board_uuid).await?;

        let db_scope = match resolved_scope {
            EmojiScope::Site => DbEmojiScope::Global,
            EmojiScope::Board => DbEmojiScope::Board,
        };

        // Check for duplicate shortcode
        let existing_count: i64 = emoji::table
            .filter(emoji::shortcode.eq(&shortcode))
            .filter(emoji::scope.eq(&db_scope))
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if existing_count > 0 {
            return Err(TinyBoardsError::from_message(
                409,
                "An emoji with this shortcode already exists in this scope",
            )
            .into());
        }

        // Upload the file and get the URL
        let image_url = upload_emoji_file(file, shortcode.clone(), user.id, ctx).await?;

        let insert_form = EmojiInsertForm {
            shortcode: shortcode.clone(),
            image_url: image_url.to_string(),
            alt_text: shortcode.clone(),
            category: category.unwrap_or_else(|| "custom".to_string()),
            scope: db_scope,
            board_id: board_uuid,
            created_by: user.id,
            is_active: true,
        };

        let new_emoji: DbEmoji = diesel::insert_into(emoji::table)
            .values(&insert_form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(EmojiObject::from(new_emoji))
    }

    /// Delete an emoji (admin/mod only)
    async fn delete_emoji(&self, ctx: &Context<'_>, emoji_id: ID) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let emoji_uuid: Uuid = emoji_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid emoji ID".into()))?;

        let existing: DbEmoji = emoji::table
            .find(emoji_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Emoji not found".into()))?;

        let scope = match existing.scope {
            DbEmojiScope::Global => EmojiScope::Site,
            DbEmojiScope::Board => EmojiScope::Board,
        };

        check_emoji_permission(conn, user, &scope, existing.board_id).await?;

        // Delete keywords first (foreign key constraint)
        diesel::delete(emoji_keywords::table.filter(emoji_keywords::emoji_id.eq(emoji_uuid)))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Delete the emoji
        let deleted = diesel::delete(emoji::table.find(emoji_uuid))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(deleted > 0)
    }
}
