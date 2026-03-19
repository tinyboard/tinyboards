use crate::{
    structs::flair::{AssignPostFlairInput, AssignUserFlairInput, PostFlair, UserFlair},
    LoggedInUser,
};
use async_graphql::*;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbFlairType,
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        flair::{
            FlairTemplate as DbFlairTemplate, PostFlair as DbPostFlair,
            PostFlairInsertForm, UserFlair as DbUserFlair, UserFlairInsertForm,
        },
    },
    schema::{board_moderators, flair_templates, post_flairs, posts, user_flairs},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

fn is_mod_or_admin_sync(
    moderator_result: Option<&BoardModerator>,
    user: &tinyboards_db::models::user::user::User,
) -> bool {
    user.is_admin
        || moderator_result
            .map(|m| m.has_permission(ModPerms::Content))
            .unwrap_or(false)
}

#[derive(Default)]
pub struct FlairAssignmentMutations;

#[Object]
impl FlairAssignmentMutations {
    /// Assign flair to a post (post author or mod/admin)
    async fn assign_post_flair(
        &self,
        ctx: &Context<'_>,
        input: AssignPostFlairInput,
    ) -> Result<PostFlair> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = input
            .post_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid post ID".into()))?;
        let template_uuid: Uuid = input
            .flair_template_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid template ID".into()))?;

        // Get post to check ownership and board
        let (post_creator_id, post_board_id): (Uuid, Uuid) = posts::table
            .find(post_uuid)
            .select((posts::creator_id, posts::board_id))
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

        let is_author = post_creator_id == user.id;

        // Check permissions if not the author
        if !is_author && !user.is_admin {
            let _mod: BoardModerator = board_moderators::table
                .filter(board_moderators::board_id.eq(post_board_id))
                .filter(board_moderators::user_id.eq(user.id))
                .filter(board_moderators::is_invite_accepted.eq(true))
                .first(conn)
                .await
                .map_err(|_| {
                    TinyBoardsError::from_message(
                        403,
                        "Only post author or moderators can assign post flairs",
                    )
                })?;
        }

        // Validate template exists, is active, is for posts, and belongs to the board
        let template: DbFlairTemplate = flair_templates::table
            .find(template_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Flair template not found".into()))?;

        if template.flair_type != DbFlairType::Post {
            return Err(TinyBoardsError::from_message(400, "Template is not for posts").into());
        }
        if !template.is_active {
            return Err(TinyBoardsError::from_message(400, "Template is not active").into());
        }
        if template.board_id != post_board_id {
            return Err(
                TinyBoardsError::from_message(400, "Template does not belong to this board")
                    .into(),
            );
        }

        // Remove existing post flair first
        diesel::delete(post_flairs::table.filter(post_flairs::post_id.eq(post_uuid)))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let form = PostFlairInsertForm {
            post_id: post_uuid,
            flair_template_id: template_uuid,
            custom_text: input.custom_text,
            custom_text_color: input.custom_text_color,
            custom_background_color: input.custom_background_color,
            assigned_by: user.id,
            is_original_author: is_author,
        };

        let flair: DbPostFlair = diesel::insert_into(post_flairs::table)
            .values(&form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(PostFlair::from(flair))
    }

    /// Remove flair from a post (post author or mod/admin)
    async fn remove_post_flair(&self, ctx: &Context<'_>, post_id: ID) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = post_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid post ID".into()))?;

        // Get post to check ownership
        let (post_creator_id, post_board_id): (Uuid, Uuid) = posts::table
            .find(post_uuid)
            .select((posts::creator_id, posts::board_id))
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

        let is_author = post_creator_id == user.id;
        if !is_author && !user.is_admin {
            let _mod: BoardModerator = board_moderators::table
                .filter(board_moderators::board_id.eq(post_board_id))
                .filter(board_moderators::user_id.eq(user.id))
                .filter(board_moderators::is_invite_accepted.eq(true))
                .first(conn)
                .await
                .map_err(|_| {
                    TinyBoardsError::from_message(403, "Insufficient permissions")
                })?;
        }

        let deleted = diesel::delete(post_flairs::table.filter(post_flairs::post_id.eq(post_uuid)))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(deleted > 0)
    }

    /// Assign flair to a user in a board (self or mod/admin)
    async fn assign_user_flair(
        &self,
        ctx: &Context<'_>,
        input: AssignUserFlairInput,
    ) -> Result<UserFlair> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let target_user_uuid: Uuid = input
            .user_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid user ID".into()))?;
        let board_uuid: Uuid = input
            .board_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;
        let template_uuid: Uuid = input
            .flair_template_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid template ID".into()))?;

        let is_self = target_user_uuid == user.id;

        // If not self-assigning, check mod/admin
        if !is_self && !user.is_admin {
            let _mod: BoardModerator = board_moderators::table
                .filter(board_moderators::board_id.eq(board_uuid))
                .filter(board_moderators::user_id.eq(user.id))
                .filter(board_moderators::is_invite_accepted.eq(true))
                .first(conn)
                .await
                .map_err(|_| {
                    TinyBoardsError::from_message(
                        403,
                        "Only the user or moderators can assign user flairs",
                    )
                })?;
        }

        // Validate template
        let template: DbFlairTemplate = flair_templates::table
            .find(template_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Flair template not found".into()))?;

        if template.flair_type != DbFlairType::User {
            return Err(TinyBoardsError::from_message(400, "Template is not for users").into());
        }
        if !template.is_active {
            return Err(TinyBoardsError::from_message(400, "Template is not active").into());
        }
        if template.board_id != board_uuid {
            return Err(
                TinyBoardsError::from_message(400, "Template does not belong to this board")
                    .into(),
            );
        }

        // Check if mod-only template
        if template.is_mod_only && is_self && !user.is_admin {
            return Err(
                TinyBoardsError::from_message(403, "This flair template is mod-only").into(),
            );
        }

        // Auto-approve if assigned by mod/admin or template doesn't require approval
        let auto_approve = !is_self || !template.is_requires_approval;

        // Remove existing user flair for this board first
        diesel::delete(
            user_flairs::table
                .filter(user_flairs::user_id.eq(target_user_uuid))
                .filter(user_flairs::board_id.eq(board_uuid)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let form = UserFlairInsertForm {
            user_id: target_user_uuid,
            board_id: board_uuid,
            flair_template_id: template_uuid,
            custom_text: input.custom_text,
            custom_text_color: input.custom_text_color,
            custom_background_color: input.custom_background_color,
            is_approved: auto_approve,
            approved_at: if auto_approve { Some(Utc::now()) } else { None },
            approved_by: if auto_approve { Some(user.id) } else { None },
            is_self_assigned: is_self,
        };

        let flair: DbUserFlair = diesel::insert_into(user_flairs::table)
            .values(&form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(UserFlair::from(flair))
    }

    /// Remove flair from a user (self or mod/admin)
    async fn remove_user_flair(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        board_id: ID,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let target_uuid: Uuid = user_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid user ID".into()))?;
        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        let is_self = target_uuid == user.id;
        if !is_self && !user.is_admin {
            let _mod: BoardModerator = board_moderators::table
                .filter(board_moderators::board_id.eq(board_uuid))
                .filter(board_moderators::user_id.eq(user.id))
                .filter(board_moderators::is_invite_accepted.eq(true))
                .first(conn)
                .await
                .map_err(|_| TinyBoardsError::from_message(403, "Insufficient permissions"))?;
        }

        let deleted = diesel::delete(
            user_flairs::table
                .filter(user_flairs::user_id.eq(target_uuid))
                .filter(user_flairs::board_id.eq(board_uuid)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(deleted > 0)
    }

    /// Approve or reject a pending user flair (mod only)
    async fn approve_user_flair(
        &self,
        ctx: &Context<'_>,
        flair_id: ID,
        approved: bool,
    ) -> Result<UserFlair> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let flair_uuid: Uuid = flair_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid flair ID".into()))?;

        let existing: DbUserFlair = user_flairs::table
            .find(flair_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("User flair not found".into()))?;

        // Check mod permissions for the board
        if !user.is_admin {
            let _mod: BoardModerator = board_moderators::table
                .filter(board_moderators::board_id.eq(existing.board_id))
                .filter(board_moderators::user_id.eq(user.id))
                .filter(board_moderators::is_invite_accepted.eq(true))
                .first(conn)
                .await
                .map_err(|_| TinyBoardsError::from_message(403, "Insufficient permissions"))?;
        }

        if approved {
            let updated: DbUserFlair = diesel::update(user_flairs::table.find(flair_uuid))
                .set((
                    user_flairs::is_approved.eq(true),
                    user_flairs::approved_at.eq(Some(Utc::now())),
                    user_flairs::approved_by.eq(Some(user.id)),
                ))
                .get_result(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
            Ok(UserFlair::from(updated))
        } else {
            // Reject = delete the flair
            diesel::delete(user_flairs::table.find(flair_uuid))
                .execute(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            Ok(UserFlair::from(existing))
        }
    }
}
