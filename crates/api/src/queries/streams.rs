use async_graphql::*;
use tinyboards_db::{
    models::stream::{
        stream::Stream as DbStream,
        stream_follower::StreamFollower as DbStreamFollower,
    },
    aggregates::structs::StreamAggregates,
    utils::DbPool,
    traits::Crud,
};
use tinyboards_utils::TinyBoardsError;

use crate::{
    LoggedInUser,
    SortType,
    structs::stream::{Stream, StreamSortType},
    structs::post::Post,
    helpers::stream::{
        permissions::can_view_stream,
        feed_generator::generate_stream_feed,
    },
};

#[derive(Default)]
pub struct StreamQueries;

#[Object]
impl StreamQueries {
    /// Get a specific stream by ID, slug, or creator username + slug
    async fn stream(
        &self,
        ctx: &Context<'_>,
        id: Option<i32>,
        slug: Option<String>,
        creator_username: Option<String>,
        share_token: Option<String>,
    ) -> Result<Stream> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.inner();

        // Fetch stream by ID, slug, or username+slug
        let stream = if let Some(stream_id) = id {
            <DbStream as Crud>::read(pool, stream_id).await
                .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Stream not found"))?
        } else if let Some(stream_slug) = slug {
            if let Some(username) = creator_username {
                DbStream::get_by_slug_and_username(pool, &stream_slug, &username).await
                    .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Stream not found"))?
            } else {
                return Err(TinyBoardsError::from_message(400, "creator_username required when using slug").into());
            }
        } else {
            return Err(TinyBoardsError::from_message(400, "Must provide id or slug").into());
        };

        // Check permissions
        can_view_stream(user, &stream, share_token.as_deref(), pool).await?;

        // Get aggregates
        let aggregates = StreamAggregates::get_for_stream(pool, stream.id).await
            .unwrap_or_default();

        Ok(Stream::from((stream, aggregates)))
    }

    /// Get stream by share token
    async fn stream_by_share_token(
        &self,
        ctx: &Context<'_>,
        token: String,
    ) -> Result<Stream> {
        let pool = ctx.data::<DbPool>()?;

        let stream = DbStream::get_by_share_token(pool, &token).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Invalid share token"))?;

        let aggregates = StreamAggregates::get_for_stream(pool, stream.id).await
            .unwrap_or_default();

        Ok(Stream::from((stream, aggregates)))
    }

    /// Get current user's own streams
    async fn my_streams(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Stream>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user()?;

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let streams = DbStream::list_for_user(pool, user.id, Some(limit), Some(offset))
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load streams"))?;

        // Get aggregates for each stream
        let mut result = Vec::new();
        for stream in streams {
            let aggregates = StreamAggregates::get_for_stream(pool, stream.id).await
                .unwrap_or_default();
            result.push(Stream::from((stream, aggregates)));
        }

        Ok(result)
    }

    /// Get streams the current user is following
    async fn followed_streams(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Stream>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user()?;

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let stream_ids = DbStreamFollower::list_followed_stream_ids(pool, user.id, Some(limit), Some(offset))
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load followed streams"))?;

        // Get full stream objects
        let mut result = Vec::new();
        for stream_id in stream_ids {
            if let Ok(stream) = <DbStream as Crud>::read(pool, stream_id).await {
                let aggregates = StreamAggregates::read(pool, stream.id).await
                    .unwrap_or_default();
                result.push(Stream::from((stream, aggregates)));
            }
        }

        Ok(result)
    }

    /// Get streams pinned to user's navbar
    async fn navbar_streams(&self, ctx: &Context<'_>) -> Result<Vec<Stream>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user()?;

        let followers = DbStreamFollower::list_navbar_streams(pool, user.id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load navbar streams"))?;

        // Get full stream objects in navbar order
        let mut result = Vec::new();
        for follower in followers {
            if let Ok(stream) = DbStream::read(pool, follower.stream_id).await {
                let aggregates = StreamAggregates::read(pool, stream.id).await
                    .unwrap_or_default();
                result.push(Stream::from((stream, aggregates)));
            }
        }

        Ok(result)
    }

    /// Discover public streams
    async fn discover_streams(
        &self,
        ctx: &Context<'_>,
        sort_by: Option<StreamSortType>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Stream>> {
        let pool = ctx.data::<DbPool>()?;

        let sort_by = sort_by.unwrap_or(StreamSortType::Popular);
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let streams = DbStream::list_public(pool, Some(format!("{:?}", sort_by).to_lowercase()), Some(limit), Some(offset))
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to discover streams"))?;

        // Get aggregates for each stream
        let mut result = Vec::new();
        for stream in streams {
            let aggregates = StreamAggregates::get_for_stream(pool, stream.id).await
                .unwrap_or_default();
            result.push(Stream::from((stream, aggregates)));
        }

        Ok(result)
    }

    /// Search streams by name or description
    async fn search_streams(
        &self,
        ctx: &Context<'_>,
        query: String,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Stream>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.inner();

        if query.trim().is_empty() {
            return Err(TinyBoardsError::from_message(400, "Search query cannot be empty").into());
        }

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let user_id = user.map(|u| u.id);

        let streams = DbStream::search(pool, &query, user_id, Some(limit), Some(offset))
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to search streams"))?;

        // Get aggregates for each stream
        let mut result = Vec::new();
        for stream in streams {
            let aggregates = StreamAggregates::get_for_stream(pool, stream.id).await
                .unwrap_or_default();
            result.push(Stream::from((stream, aggregates)));
        }

        Ok(result)
    }

    /// Get posts for a stream (CRITICAL: combines flair and board subscriptions)
    async fn stream_posts(
        &self,
        ctx: &Context<'_>,
        stream_id: Option<i32>,
        slug: Option<String>,
        creator_username: Option<String>,
        share_token: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
        sort_type: Option<SortType>,
    ) -> Result<Vec<Post>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.inner();

        // Fetch stream
        let stream = if let Some(sid) = stream_id {
            <DbStream as Crud>::read(pool, sid).await
                .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Stream not found"))?
        } else if let Some(stream_slug) = slug {
            if let Some(username) = creator_username {
                DbStream::get_by_slug_and_username(pool, &stream_slug, &username).await
                    .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Stream not found"))?
            } else {
                return Err(TinyBoardsError::from_message(400, "creator_username required when using slug").into());
            }
        } else {
            return Err(TinyBoardsError::from_message(400, "Must provide stream_id or slug").into());
        };

        // Check permissions
        can_view_stream(user, &stream, share_token.as_deref(), pool).await?;

        // Update last_viewed_at if user is authenticated
        if let Some(u) = user {
            let _ = DbStream::update_last_viewed(pool, stream.id, u.id).await;
        }

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);
        let sort = sort_type.unwrap_or(SortType::Hot);

        // Generate feed using the feed generator helper
        let posts = generate_stream_feed(
            pool,
            &stream,
            user,
            sort.into(),
            limit,
            offset,
        ).await?;

        Ok(posts)
    }
}
