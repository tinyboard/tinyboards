use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::message::message::{PrivateMessage as DbPrivateMessage, PrivateMessageUpdateForm},
    schema::{private_messages, site},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{
    structs::message::PrivateMessage,
    LoggedInUser,
    utils::emoji::process_content_with_emojis,
    Settings,
};

#[derive(Default)]
pub struct EditMessageMutations;

#[derive(InputObject)]
pub struct EditMessageInput {
    pub message_id: ID,
    pub subject: Option<String>,
    pub body: Option<String>,
}

#[derive(SimpleObject)]
pub struct EditMessageResponse {
    pub message: PrivateMessage,
}

#[Object]
impl EditMessageMutations {
    /// Edit a private message (only sender can edit)
    pub async fn edit_message(
        &self,
        ctx: &Context<'_>,
        input: EditMessageInput,
    ) -> Result<EditMessageResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;
        let conn = &mut get_conn(pool).await?;

        let message_id: Uuid = input.message_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid message ID".into()))?;

        // Check if message exists and user owns it
        let existing: DbPrivateMessage = private_messages::table
            .find(message_id)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Message not found".into()))?;

        if existing.creator_id != user.id {
            return Err(TinyBoardsError::from_message(403, "You can only edit your own messages").into());
        }

        if existing.deleted_at.is_some() {
            return Err(TinyBoardsError::from_message(404, "Message has been deleted").into());
        }

        // Validate input if provided
        if let Some(ref subject) = input.subject {
            if subject.len() > 200 {
                return Err(TinyBoardsError::from_message(400, "Subject too long").into());
            }
        }

        if let Some(ref body) = input.body {
            if body.trim().is_empty() {
                return Err(TinyBoardsError::from_message(400, "Message body cannot be empty").into());
            }
            if body.len() > 10000 {
                return Err(TinyBoardsError::from_message(400, "Message too long").into());
            }
        }

        // Process body HTML if body is being updated
        let body_html = if let Some(ref body) = input.body {
            let settings = ctx.data::<Settings>()?.as_ref();

            let site_config: tinyboards_db::models::site::site::Site = site::table
                .first(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            let emoji_limit = if site_config.emoji_enabled {
                site_config.max_emojis_per_comment.map(|limit| limit as usize)
            } else {
                Some(0)
            };

            Some(
                process_content_with_emojis(
                    body,
                    pool,
                    None,
                    settings,
                    emoji_limit,
                )
                .await?,
            )
        } else {
            None
        };

        let update_form = PrivateMessageUpdateForm {
            subject: input.subject.map(Some),
            body: input.body,
            body_html,
            is_read: None,
            is_sender_hidden: None,
            updated_at: Some(chrono::Utc::now()),
            deleted_at: None,
        };

        let updated: DbPrivateMessage = diesel::update(private_messages::table.find(message_id))
            .set(&update_form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(EditMessageResponse {
            message: PrivateMessage::from(updated),
        })
    }
}
