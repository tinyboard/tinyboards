use crate::{
    LoggedInUser,
    structs::reaction::{BoardReactionSettings as GqlBoardReactionSettings, Reaction as GqlReaction},
};
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        reaction::{
            BoardReactionSettings as DbBoardReactionSettings,
            BoardReactionSettingsInsertForm,
            BoardReactionSettingsUpdateForm,
            Reaction as DbReaction,
            ReactionInsertForm,
        },
        user::user::AdminPerms,
    },
    schema::{board_moderators, board_reaction_settings, comments, posts, reactions},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

#[derive(InputObject)]
pub struct AddReactionInput {
    pub post_id: Option<ID>,
    pub comment_id: Option<ID>,
    pub emoji: String,
}

#[derive(SimpleObject)]
pub struct AddReactionResponse {
    pub reaction: GqlReaction,
}

#[derive(InputObject)]
pub struct RemoveReactionInput {
    pub post_id: Option<ID>,
    pub comment_id: Option<ID>,
    pub emoji: String,
}

#[derive(SimpleObject)]
pub struct RemoveReactionResponse {
    pub success: bool,
}

#[derive(InputObject)]
pub struct UpdateBoardReactionSettingsInput {
    pub board_id: ID,
    pub emoji_weights: Option<Json<serde_json::Value>>,
    pub is_reactions_enabled: Option<bool>,
    /// JSON array of reaction emoji entries. Each entry:
    /// `{"type":"unicode","value":"👍"}` or `{"type":"custom","shortcode":"party_parrot","imageUrl":"https://..."}`
    /// Empty array means "use site defaults".
    pub reaction_emojis: Option<Json<serde_json::Value>>,
}

#[derive(SimpleObject)]
pub struct UpdateBoardReactionSettingsResponse {
    pub settings: GqlBoardReactionSettings,
}

#[derive(Default)]
pub struct ReactionMutations;

#[Object]
impl ReactionMutations {
    /// Add a reaction to a post or comment (BUG-008 fix: uses get_conn instead of pool.get().await.unwrap())
    async fn add_reaction(
        &self,
        ctx: &Context<'_>,
        input: AddReactionInput,
    ) -> Result<AddReactionResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        // Validate input: must have either post_id or comment_id, not both
        if input.post_id.is_none() && input.comment_id.is_none() {
            return Err(
                TinyBoardsError::from_message(400, "Must provide either post_id or comment_id")
                    .into(),
            );
        }
        if input.post_id.is_some() && input.comment_id.is_some() {
            return Err(
                TinyBoardsError::from_message(400, "Cannot react to both post and comment").into(),
            );
        }

        let post_uuid: Option<Uuid> = if let Some(ref pid) = input.post_id {
            Some(
                pid.parse()
                    .map_err(|_| TinyBoardsError::NotFound("Invalid post ID".into()))?,
            )
        } else {
            None
        };

        let comment_uuid: Option<Uuid> = if let Some(ref cid) = input.comment_id {
            Some(
                cid.parse()
                    .map_err(|_| TinyBoardsError::NotFound("Invalid comment ID".into()))?,
            )
        } else {
            None
        };

        // Get board_id to check reaction settings
        let board_id: Uuid = if let Some(pid) = post_uuid {
            posts::table
                .find(pid)
                .select(posts::board_id)
                .first(conn)
                .await
                .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?
        } else if let Some(cid) = comment_uuid {
            comments::table
                .find(cid)
                .select(comments::board_id)
                .first(conn)
                .await
                .map_err(|_| TinyBoardsError::NotFound("Comment not found".into()))?
        } else {
            return Err(TinyBoardsError::from_message(500, "Invalid state").into());
        };

        // Get board reaction settings
        let settings: Option<DbBoardReactionSettings> = board_reaction_settings::table
            .filter(board_reaction_settings::board_id.eq(board_id))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Default: reactions enabled with equal weight
        let (is_enabled, emoji_weights) = match settings {
            Some(ref s) => (s.is_reactions_enabled, &s.emoji_weights),
            None => {
                // No settings = reactions enabled by default
                (true, &serde_json::Value::Object(serde_json::Map::new()))
            }
        };

        if !is_enabled {
            return Err(
                TinyBoardsError::from_message(403, "Reactions are disabled for this board").into(),
            );
        }

        // Get score from emoji weights (default to 1 if not configured)
        let score: i32 = emoji_weights
            .as_object()
            .and_then(|obj| obj.get(&input.emoji))
            .and_then(|v| v.as_i64())
            .unwrap_or(1) as i32;

        // Check if user already reacted with this emoji
        let mut existing_query = reactions::table
            .filter(reactions::user_id.eq(user.id))
            .filter(reactions::emoji.eq(&input.emoji))
            .into_boxed();

        if let Some(pid) = post_uuid {
            existing_query = existing_query.filter(reactions::post_id.eq(pid));
        }
        if let Some(cid) = comment_uuid {
            existing_query = existing_query.filter(reactions::comment_id.eq(cid));
        }

        let existing_count: i64 = existing_query
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if existing_count > 0 {
            return Err(
                TinyBoardsError::from_message(409, "You already reacted with this emoji").into(),
            );
        }

        // Create reaction
        let reaction_form = ReactionInsertForm {
            user_id: user.id,
            post_id: post_uuid,
            comment_id: comment_uuid,
            emoji: input.emoji,
            score,
        };

        let reaction: DbReaction = diesel::insert_into(reactions::table)
            .values(&reaction_form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(AddReactionResponse {
            reaction: GqlReaction::from(reaction),
        })
    }

    /// Remove a reaction from a post or comment (BUG-008 fix)
    async fn remove_reaction(
        &self,
        ctx: &Context<'_>,
        input: RemoveReactionInput,
    ) -> Result<RemoveReactionResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        // Validate input
        if input.post_id.is_none() && input.comment_id.is_none() {
            return Err(
                TinyBoardsError::from_message(400, "Must provide either post_id or comment_id")
                    .into(),
            );
        }

        // Delete the reaction based on post or comment
        let deleted: usize = if let Some(ref pid) = input.post_id {
            let post_uuid: Uuid = pid
                .parse()
                .map_err(|_| TinyBoardsError::NotFound("Invalid post ID".into()))?;

            diesel::delete(
                reactions::table
                    .filter(reactions::user_id.eq(user.id))
                    .filter(reactions::emoji.eq(&input.emoji))
                    .filter(reactions::post_id.eq(post_uuid)),
            )
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?
        } else if let Some(ref cid) = input.comment_id {
            let comment_uuid: Uuid = cid
                .parse()
                .map_err(|_| TinyBoardsError::NotFound("Invalid comment ID".into()))?;

            diesel::delete(
                reactions::table
                    .filter(reactions::user_id.eq(user.id))
                    .filter(reactions::emoji.eq(&input.emoji))
                    .filter(reactions::comment_id.eq(comment_uuid)),
            )
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?
        } else {
            return Err(TinyBoardsError::from_message(400, "Must provide either post_id or comment_id").into());
        };

        if deleted == 0 {
            return Err(TinyBoardsError::from_message(404, "Reaction not found").into());
        }

        Ok(RemoveReactionResponse { success: true })
    }

    /// Update board reaction settings (moderators/admin only, BUG-008 fix)
    async fn update_board_reaction_settings(
        &self,
        ctx: &Context<'_>,
        input: UpdateBoardReactionSettingsInput,
    ) -> Result<UpdateBoardReactionSettingsResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let board_uuid: Uuid = input
            .board_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        // Check permissions
        let is_admin = user.has_permission(AdminPerms::Boards);
        if !is_admin {
            let moderator: BoardModerator = board_moderators::table
                .filter(board_moderators::board_id.eq(board_uuid))
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
                    "Insufficient permissions to update reaction settings",
                )
                .into());
            }
        }

        // Validate emoji_weights if provided
        if let Some(ref weights) = input.emoji_weights {
            if let Some(obj) = weights.0.as_object() {
                for value in obj.values() {
                    if let Some(score) = value.as_i64() {
                        if !(-1..=1).contains(&score) {
                            return Err(TinyBoardsError::from_message(
                                400,
                                "Emoji weights must be -1, 0, or 1",
                            )
                            .into());
                        }
                    } else {
                        return Err(TinyBoardsError::from_message(
                            400,
                            "Emoji weights must be integers",
                        )
                        .into());
                    }
                }
            }
        }

        // Validate reaction_emojis if provided
        if let Some(ref emojis) = input.reaction_emojis {
            let arr = emojis.0.as_array().ok_or_else(|| {
                TinyBoardsError::from_message(400, "reaction_emojis must be a JSON array")
            })?;
            if arr.len() > 10 {
                return Err(TinyBoardsError::from_message(
                    400,
                    "reaction_emojis can have at most 10 entries",
                )
                .into());
            }
            for entry in arr {
                let obj = entry.as_object().ok_or_else(|| {
                    TinyBoardsError::from_message(400, "Each reaction emoji must be a JSON object")
                })?;
                let emoji_type = obj
                    .get("type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        TinyBoardsError::from_message(400, "Each reaction emoji must have a 'type' field (\"unicode\" or \"custom\")")
                    })?;
                match emoji_type {
                    "unicode" => {
                        if obj.get("value").and_then(|v| v.as_str()).is_none() {
                            return Err(TinyBoardsError::from_message(
                                400,
                                "Unicode reaction emoji must have a 'value' field",
                            )
                            .into());
                        }
                    }
                    "custom" => {
                        if obj.get("shortcode").and_then(|v| v.as_str()).is_none() {
                            return Err(TinyBoardsError::from_message(
                                400,
                                "Custom reaction emoji must have a 'shortcode' field",
                            )
                            .into());
                        }
                        if obj.get("imageUrl").and_then(|v| v.as_str()).is_none() {
                            return Err(TinyBoardsError::from_message(
                                400,
                                "Custom reaction emoji must have an 'imageUrl' field",
                            )
                            .into());
                        }
                    }
                    _ => {
                        return Err(TinyBoardsError::from_message(
                            400,
                            "Reaction emoji type must be \"unicode\" or \"custom\"",
                        )
                        .into());
                    }
                }
            }
        }

        // Check if settings already exist
        let existing: Option<DbBoardReactionSettings> = board_reaction_settings::table
            .filter(board_reaction_settings::board_id.eq(board_uuid))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let settings: DbBoardReactionSettings = if let Some(existing_settings) = existing {
            // Update existing settings
            let update_form = BoardReactionSettingsUpdateForm {
                emoji_weights: input.emoji_weights.map(|j| j.0),
                is_reactions_enabled: input.is_reactions_enabled,
                reaction_emojis: input.reaction_emojis.map(|j| j.0),
            };

            diesel::update(
                board_reaction_settings::table.find(existing_settings.id),
            )
            .set(&update_form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?
        } else {
            // Create new settings
            let insert_form = BoardReactionSettingsInsertForm {
                board_id: board_uuid,
                emoji_weights: input
                    .emoji_weights
                    .map(|j| j.0)
                    .unwrap_or_else(|| serde_json::json!({})),
                is_reactions_enabled: input.is_reactions_enabled.unwrap_or(true),
                reaction_emojis: input
                    .reaction_emojis
                    .map(|j| j.0)
                    .unwrap_or_else(|| serde_json::json!([])),
            };

            diesel::insert_into(board_reaction_settings::table)
                .values(&insert_form)
                .get_result(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?
        };

        Ok(UpdateBoardReactionSettingsResponse {
            settings: GqlBoardReactionSettings::from(settings),
        })
    }
}
