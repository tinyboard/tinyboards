use async_graphql::*;
use tinyboards_db::{
    models::stream::{
        stream::{Stream as DbStream, CreateStreamForm},
    },
    aggregates::structs::StreamAggregates,
    utils::DbPool,
    traits::Crud,
};
use tinyboards_utils::TinyBoardsError;

use crate::{
    LoggedInUser,
    structs::stream::{Stream, CreateStreamInput, UpdateStreamInput},
    helpers::stream::permissions::{
        can_edit_stream,
        can_delete_stream,
        check_stream_quota,
    },
};

#[derive(Default)]
pub struct StreamManageMutations;

#[Object]
impl StreamManageMutations {
    /// Create a new stream
    async fn create_stream(
        &self,
        ctx: &Context<'_>,
        input: CreateStreamInput,
    ) -> Result<Stream> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Check stream quota
        check_stream_quota(user, pool).await?;

        // Validate input
        if input.name.trim().is_empty() {
            return Err(TinyBoardsError::from_message(400, "Stream name cannot be empty").into());
        }

        if input.name.len() > 100 {
            return Err(TinyBoardsError::from_message(400, "Stream name must be 100 characters or less").into());
        }

        if let Some(ref desc) = input.description {
            if desc.len() > 500 {
                return Err(TinyBoardsError::from_message(400, "Stream description must be 500 characters or less").into());
            }
        }

        if let Some(max_posts) = input.max_posts_per_board {
            if max_posts < 1 || max_posts > 100 {
                return Err(TinyBoardsError::from_message(400, "max_posts_per_board must be between 1 and 100").into());
            }
        }

        // Generate unique slug
        let slug = generate_slug(&input.name, user.id, pool).await?;

        // Create stream
        let form = CreateStreamForm {
            name: input.name.trim().to_string(),
            slug,
            description: input.description.map(|d| d.trim().to_string()),
            creator_id: user.id,
            is_public: input.is_public.unwrap_or(false),
            is_discoverable: input.is_public.unwrap_or(false), // Discoverable if public by default
            share_token: None,
            max_posts_per_board: input.max_posts_per_board,
            icon: None,
            color: None,
            sort_type: "hot".to_string(),
            time_range: None,
            show_nsfw: false,
        };

        let stream = <DbStream as Crud>::create(pool, &form)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to create stream"))?;

        let aggregates = StreamAggregates::get_for_stream(pool, stream.id).await.unwrap_or_default();

        Ok(Stream::from((stream, aggregates)))
    }

    /// Update an existing stream
    async fn update_stream(
        &self,
        ctx: &Context<'_>,
        input: UpdateStreamInput,
    ) -> Result<Stream> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Get existing stream
        let stream = <DbStream as Crud>::read(pool, input.stream_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Stream not found"))?;

        // Check permissions
        can_edit_stream(Some(user), &stream)?;

        // Validate input
        if let Some(ref name) = input.name {
            if name.trim().is_empty() {
                return Err(TinyBoardsError::from_message(400, "Stream name cannot be empty").into());
            }
            if name.len() > 100 {
                return Err(TinyBoardsError::from_message(400, "Stream name must be 100 characters or less").into());
            }
        }

        if let Some(ref desc) = input.description {
            if desc.len() > 500 {
                return Err(TinyBoardsError::from_message(400, "Stream description must be 500 characters or less").into());
            }
        }

        if let Some(max_posts) = input.max_posts_per_board {
            if max_posts < 1 || max_posts > 100 {
                return Err(TinyBoardsError::from_message(400, "max_posts_per_board must be between 1 and 100").into());
            }
        }

        // Create update form with current values as base
        let form = CreateStreamForm {
            creator_id: stream.creator_id,
            name: input.name.map(|n| n.trim().to_string()).unwrap_or(stream.name),
            slug: stream.slug,
            description: input.description.or(stream.description),
            icon: stream.icon,
            color: stream.color,
            is_public: input.is_public.unwrap_or(stream.is_public),
            is_discoverable: stream.is_discoverable,
            share_token: stream.share_token,
            sort_type: stream.sort_type,
            time_range: stream.time_range,
            show_nsfw: stream.show_nsfw,
            max_posts_per_board: input.max_posts_per_board.or(stream.max_posts_per_board),
        };

        // Update stream
        let updated_stream = <DbStream as Crud>::update(pool, input.stream_id, &form)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update stream"))?;

        let aggregates = StreamAggregates::get_for_stream(pool, updated_stream.id).await.unwrap_or_default();

        Ok(Stream::from((updated_stream, aggregates)))
    }

    /// Delete a stream
    async fn delete_stream(
        &self,
        ctx: &Context<'_>,
        stream_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Get stream
        let stream = <DbStream as Crud>::read(pool, stream_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Stream not found"))?;

        // Check permissions
        can_delete_stream(Some(user), &stream)?;

        // Delete stream (cascades to subscriptions and followers)
        <DbStream as Crud>::delete(pool, stream_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to delete stream"))?;

        Ok(true)
    }

    /// Generate a share token for a stream
    async fn generate_stream_share_token(
        &self,
        ctx: &Context<'_>,
        stream_id: i32,
    ) -> Result<String> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Get stream
        let stream = <DbStream as Crud>::read(pool, stream_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Stream not found"))?;

        // Check permissions
        can_edit_stream(Some(user), &stream)?;

        // Generate cryptographically secure token
        let token = generate_share_token();

        // Update stream with new token
        DbStream::update_share_token(pool, stream_id, Some(&token))
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to generate share token"))?;

        Ok(token)
    }

    /// Revoke the share token for a stream
    async fn revoke_stream_share_token(
        &self,
        ctx: &Context<'_>,
        stream_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Get stream
        let stream = <DbStream as Crud>::read(pool, stream_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Stream not found"))?;

        // Check permissions
        can_edit_stream(Some(user), &stream)?;

        // Remove token
        DbStream::update_share_token(pool, stream_id, None)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to revoke share token"))?;

        Ok(true)
    }

    /// Mark a stream as viewed (updates last_viewed_at)
    async fn mark_stream_viewed(
        &self,
        ctx: &Context<'_>,
        stream_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Update last viewed timestamp
        DbStream::update_last_viewed(pool, stream_id, user.id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update view timestamp"))?;

        Ok(true)
    }
}

/// Generate a URL-safe slug from a stream name
async fn generate_slug(name: &str, user_id: i32, pool: &DbPool) -> Result<String, async_graphql::Error> {
    use slug::slugify;

    let base_slug = slugify(name);

    // Ensure slug is not empty
    let base_slug = if base_slug.is_empty() {
        "stream".to_string()
    } else {
        base_slug
    };

    // Check if slug is unique for this user
    let mut slug = base_slug.clone();
    let mut counter = 1;

    while DbStream::slug_exists_for_user(pool, user_id, &slug).await.unwrap_or(false) {
        slug = format!("{}-{}", base_slug, counter);
        counter += 1;

        // Prevent infinite loops
        if counter > 1000 {
            return Err(TinyBoardsError::from_message(500, "Failed to generate unique slug").into());
        }
    }

    Ok(slug)
}

/// Generate a cryptographically secure share token (64 characters)
fn generate_share_token() -> String {
    use rand::Rng;

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    const TOKEN_LENGTH: usize = 64;

    let mut rng = rand::thread_rng();

    (0..TOKEN_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
