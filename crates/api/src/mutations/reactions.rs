use crate::{LoggedInUser, structs::reaction::{Reaction as GqlReaction, BoardReactionSettings as GqlBoardReactionSettings}};
use async_graphql::*;
use tinyboards_db::{
    models::reaction::reactions::{
        Reaction as DbReaction,
        ReactionForm,
        BoardReactionSettings as DbBoardReactionSettings,
        BoardReactionSettingsForm,
    },
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

#[derive(InputObject)]
pub struct AddReactionInput {
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub emoji: String,
}

#[derive(SimpleObject)]
pub struct AddReactionResponse {
    pub reaction: GqlReaction,
}

#[derive(InputObject)]
pub struct RemoveReactionInput {
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub emoji: String,
}

#[derive(SimpleObject)]
pub struct RemoveReactionResponse {
    pub success: bool,
}

#[derive(InputObject)]
pub struct UpdateBoardReactionSettingsInput {
    pub board_id: i32,
    pub emoji_weights: Option<Json<serde_json::Value>>,
    pub reactions_enabled: Option<bool>,
}

#[derive(SimpleObject)]
pub struct UpdateBoardReactionSettingsResponse {
    pub settings: GqlBoardReactionSettings,
}

#[derive(Default)]
pub struct ReactionMutations;

#[Object]
impl ReactionMutations {
    /// Add a reaction to a post or comment
    async fn add_reaction(
        &self,
        ctx: &Context<'_>,
        input: AddReactionInput,
    ) -> Result<AddReactionResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Validate input: must have either post_id or comment_id, not both
        if input.post_id.is_none() && input.comment_id.is_none() {
            return Err(TinyBoardsError::from_message(400, "Must provide either post_id or comment_id").into());
        }
        if input.post_id.is_some() && input.comment_id.is_some() {
            return Err(TinyBoardsError::from_message(400, "Cannot react to both post and comment").into());
        }

        // Get board_id to check reaction settings
        let board_id = if let Some(pid) = input.post_id {
            use tinyboards_db::models::post::posts::Post;
            let post = Post::read(pool, pid).await
                .map_err(|_| TinyBoardsError::from_message(404, "Post not found"))?;
            post.board_id
        } else if let Some(cid) = input.comment_id {
            use tinyboards_db::models::comment::comments::Comment;
            let comment = Comment::read(pool, cid).await
                .map_err(|_| TinyBoardsError::from_message(404, "Comment not found"))?;
            comment.board_id
        } else {
            return Err(TinyBoardsError::from_message(500, "Invalid state").into());
        };

        // Get board reaction settings
        let settings = DbBoardReactionSettings::get_for_board(pool, board_id).await
            .map_err(|_| TinyBoardsError::from_message(500, "Failed to load reaction settings"))?;

        // Check if reactions are enabled
        if !settings.reactions_enabled {
            return Err(TinyBoardsError::from_message(403, "Reactions are disabled for this board").into());
        }

        // Get emoji weights map
        let weights = settings.emoji_weights.as_object()
            .ok_or_else(|| TinyBoardsError::from_message(500, "Invalid emoji weights configuration"))?;

        // Check if emoji is allowed and get its score
        let score = weights.get(&input.emoji)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| TinyBoardsError::from_message(400, "This emoji is not allowed on this board"))? as i32;

        // Check if user already reacted with this emoji
        let existing = DbReaction::get_user_reaction(
            pool,
            user.id,
            input.post_id,
            input.comment_id,
            &input.emoji,
        ).await;

        if existing.is_ok() {
            return Err(TinyBoardsError::from_message(409, "You already reacted with this emoji").into());
        }

        // Create reaction
        let reaction_form = ReactionForm {
            user_id: user.id,
            post_id: input.post_id,
            comment_id: input.comment_id,
            emoji: input.emoji.clone(),
            score,
            creation_date: None,
        };

        let reaction = DbReaction::create(pool, &reaction_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to create reaction"))?;

        Ok(AddReactionResponse {
            reaction: GqlReaction::from(reaction),
        })
    }

    /// Remove a reaction from a post or comment
    async fn remove_reaction(
        &self,
        ctx: &Context<'_>,
        input: RemoveReactionInput,
    ) -> Result<RemoveReactionResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Validate input
        if input.post_id.is_none() && input.comment_id.is_none() {
            return Err(TinyBoardsError::from_message(400, "Must provide either post_id or comment_id").into());
        }

        // Delete the reaction
        let deleted = DbReaction::delete_user_reaction(
            pool,
            user.id,
            input.post_id,
            input.comment_id,
            &input.emoji,
        ).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to remove reaction"))?;

        if deleted == 0 {
            return Err(TinyBoardsError::from_message(404, "Reaction not found").into());
        }

        Ok(RemoveReactionResponse {
            success: true,
        })
    }

    /// Update board reaction settings (moderators only)
    async fn update_board_reaction_settings(
        &self,
        ctx: &Context<'_>,
        input: UpdateBoardReactionSettingsInput,
    ) -> Result<UpdateBoardReactionSettingsResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Check if user is moderator or admin
        use tinyboards_db::models::board::board_mods::BoardModerator;
        use tinyboards_db::models::user::user::AdminPerms;

        let is_admin = user.has_permission(AdminPerms::Boards);
        let is_mod = if !is_admin {
            BoardModerator::get_by_user_id_for_board(pool, user.id, input.board_id, true).await.is_ok()
        } else {
            true
        };

        if !is_mod && !is_admin {
            return Err(TinyBoardsError::from_message(403, "You must be a moderator or admin to update reaction settings").into());
        }

        // Validate emoji_weights if provided
        if let Some(ref weights) = input.emoji_weights {
            if let Some(obj) = weights.0.as_object() {
                for value in obj.values() {
                    if let Some(score) = value.as_i64() {
                        if score < -1 || score > 1 {
                            return Err(TinyBoardsError::from_message(400, "Emoji weights must be -1, 0, or 1").into());
                        }
                    } else {
                        return Err(TinyBoardsError::from_message(400, "Emoji weights must be integers").into());
                    }
                }
            }
        }

        // Get existing settings
        let existing = DbBoardReactionSettings::get_for_board(pool, input.board_id).await
            .map_err(|_| TinyBoardsError::from_message(404, "Board reaction settings not found"))?;

        // Update settings
        let form = BoardReactionSettingsForm {
            board_id: input.board_id,
            emoji_weights: input.emoji_weights.map(|j| j.0),
            reactions_enabled: input.reactions_enabled,
        };

        let updated = DbBoardReactionSettings::update(pool, existing.id, &form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update reaction settings"))?;

        Ok(UpdateBoardReactionSettingsResponse {
            settings: GqlBoardReactionSettings::from(updated),
        })
    }
}
