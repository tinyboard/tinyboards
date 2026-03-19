use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbSortType,
    models::stream::{Stream as DbStream, StreamFollowerInsertForm, StreamInsertForm, StreamUpdateForm},
    schema::{stream_followers, streams},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{
    structs::stream::{CreateStreamInput, Stream, UpdateStreamInput},
    LoggedInUser,
};

#[derive(Default)]
pub struct StreamManageMutations;

#[Object]
impl StreamManageMutations {
    /// Create a new stream
    async fn create_stream(&self, ctx: &Context<'_>, input: CreateStreamInput) -> Result<Stream> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        // Validate
        if input.name.trim().is_empty() {
            return Err(TinyBoardsError::from_message(400, "Stream name cannot be empty").into());
        }
        if input.name.len() > 100 {
            return Err(
                TinyBoardsError::from_message(400, "Stream name must be 100 characters or less")
                    .into(),
            );
        }

        // Check stream quota (max 25 per user)
        let existing_count: i64 = streams::table
            .filter(streams::creator_id.eq(user.id))
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if existing_count >= 25 {
            return Err(
                TinyBoardsError::from_message(403, "Maximum of 25 streams per user").into(),
            );
        }

        // Generate slug
        let base_slug = input
            .name
            .trim()
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>();
        let base_slug = if base_slug.is_empty() {
            "stream".to_string()
        } else {
            base_slug
        };

        let mut slug = base_slug.clone();
        let mut counter = 1;
        while streams::table
            .filter(streams::creator_id.eq(user.id))
            .filter(streams::slug.eq(&slug))
            .count()
            .get_result::<i64>(conn)
            .await
            .unwrap_or(0)
            > 0
        {
            slug = format!("{}-{}", base_slug, counter);
            counter += 1;
            if counter > 100 {
                return Err(
                    TinyBoardsError::from_message(500, "Failed to generate unique slug").into(),
                );
            }
        }

        let sort_type = match input.sort_type.as_deref() {
            Some("new") => DbSortType::New,
            Some("top") => DbSortType::Top,
            Some("old") => DbSortType::Old,
            Some("most_comments") => DbSortType::MostComments,
            Some("controversial") => DbSortType::Controversial,
            _ => DbSortType::Hot,
        };

        let form = StreamInsertForm {
            creator_id: user.id,
            name: input.name.trim().to_string(),
            slug,
            description: input.description,
            icon: None,
            color: None,
            is_public: input.is_public.unwrap_or(false),
            is_discoverable: input.is_discoverable.unwrap_or(false),
            share_token: None,
            sort_type,
            time_range: input.time_range,
            show_nsfw: input.show_nsfw.unwrap_or(false),
            max_posts_per_board: input.max_posts_per_board,
        };

        let stream: DbStream = diesel::insert_into(streams::table)
            .values(&form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Auto-follow the stream for the creator
        let follower_form = StreamFollowerInsertForm {
            stream_id: stream.id,
            user_id: user.id,
            added_to_navbar: false,
            navbar_position: None,
        };
        let _ = diesel::insert_into(stream_followers::table)
            .values(&follower_form)
            .execute(conn)
            .await;

        Ok(Stream::from(stream))
    }

    /// Update an existing stream (creator only)
    async fn update_stream(
        &self,
        ctx: &Context<'_>,
        stream_id: ID,
        input: UpdateStreamInput,
    ) -> Result<Stream> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let stream_uuid: Uuid = stream_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid stream ID".into()))?;

        let existing: DbStream = streams::table
            .find(stream_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Stream not found".into()))?;

        if existing.creator_id != user.id && !user.is_admin {
            return Err(TinyBoardsError::from_message(403, "Only the stream creator can edit it").into());
        }

        let sort_type = input.sort_type.as_deref().map(|s| match s {
            "new" => DbSortType::New,
            "top" => DbSortType::Top,
            "old" => DbSortType::Old,
            "most_comments" => DbSortType::MostComments,
            "controversial" => DbSortType::Controversial,
            _ => DbSortType::Hot,
        });

        let form = StreamUpdateForm {
            name: input.name,
            slug: None,
            description: input.description.map(Some),
            icon: input.icon.map(Some),
            color: input.color.map(Some),
            is_public: input.is_public,
            is_discoverable: input.is_discoverable,
            share_token: None,
            sort_type,
            time_range: input.time_range.map(Some),
            show_nsfw: input.show_nsfw,
            max_posts_per_board: input.max_posts_per_board.map(Some),
            last_viewed_at: None,
        };

        let updated: DbStream = diesel::update(streams::table.find(stream_uuid))
            .set(&form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(Stream::from(updated))
    }

    /// Delete a stream (creator or admin)
    async fn delete_stream(&self, ctx: &Context<'_>, stream_id: ID) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let stream_uuid: Uuid = stream_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid stream ID".into()))?;

        let existing: DbStream = streams::table
            .find(stream_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Stream not found".into()))?;

        if existing.creator_id != user.id && !user.is_admin {
            return Err(
                TinyBoardsError::from_message(403, "Only the stream creator or admin can delete it")
                    .into(),
            );
        }

        let deleted = diesel::delete(streams::table.find(stream_uuid))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(deleted > 0)
    }

    /// Generate a share token for a stream
    async fn regenerate_share_token(&self, ctx: &Context<'_>, stream_id: ID) -> Result<String> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let stream_uuid: Uuid = stream_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid stream ID".into()))?;

        let existing: DbStream = streams::table
            .find(stream_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Stream not found".into()))?;

        if existing.creator_id != user.id {
            return Err(TinyBoardsError::from_message(403, "Only the creator can manage share tokens").into());
        }

        let token = Uuid::new_v4().to_string().replace('-', "");

        diesel::update(streams::table.find(stream_uuid))
            .set(streams::share_token.eq(Some(&token)))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(token)
    }
}
