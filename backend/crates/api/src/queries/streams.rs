use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::stream::Stream as DbStream,
    schema::{stream_followers, streams},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{
    structs::stream::{Stream, StreamSortType},
    LoggedInUser,
};

#[derive(Default)]
pub struct StreamQueries;

#[Object]
impl StreamQueries {
    /// Get a specific stream by ID or by slug
    async fn stream(
        &self,
        ctx: &Context<'_>,
        id: Option<ID>,
        slug: Option<String>,
    ) -> Result<Stream> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.inner();
        let conn = &mut get_conn(pool).await?;

        let stream: DbStream = if let Some(stream_id) = id {
            let stream_uuid: Uuid = stream_id
                .parse()
                .map_err(|_| TinyBoardsError::NotFound("Invalid stream ID".into()))?;

            streams::table
                .find(stream_uuid)
                .first(conn)
                .await
                .map_err(|_| TinyBoardsError::NotFound("Stream not found".into()))?
        } else if let Some(stream_slug) = slug {
            streams::table
                .filter(streams::slug.eq(&stream_slug))
                .first(conn)
                .await
                .map_err(|_| TinyBoardsError::NotFound("Stream not found".into()))?
        } else {
            return Err(TinyBoardsError::from_message(400, "Must provide id or slug").into());
        };

        // Check visibility: private streams only visible to creator or followers
        if !stream.is_public {
            let has_access = if let Some(u) = user {
                if u.id == stream.creator_id || u.is_admin {
                    true
                } else {
                    stream_followers::table
                        .filter(stream_followers::stream_id.eq(stream.id))
                        .filter(stream_followers::user_id.eq(u.id))
                        .count()
                        .get_result::<i64>(conn)
                        .await
                        .unwrap_or(0)
                        > 0
                }
            } else {
                false
            };

            if !has_access {
                return Err(
                    TinyBoardsError::from_message(403, "This stream is private").into(),
                );
            }
        }

        Ok(Stream::from(stream))
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
        let conn = &mut get_conn(pool).await?;

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let results: Vec<DbStream> = streams::table
            .filter(streams::creator_id.eq(user.id))
            .order(streams::updated_at.desc())
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Stream::from).collect())
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
        let conn = &mut get_conn(pool).await?;

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let results: Vec<DbStream> = streams::table
            .inner_join(stream_followers::table.on(stream_followers::stream_id.eq(streams::id)))
            .filter(stream_followers::user_id.eq(user.id))
            .select(streams::all_columns)
            .order(streams::updated_at.desc())
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Stream::from).collect())
    }

    /// Get streams pinned to user's navbar, ordered by position
    async fn navbar_streams(&self, ctx: &Context<'_>) -> Result<Vec<Stream>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user()?;
        let conn = &mut get_conn(pool).await?;

        let results: Vec<DbStream> = streams::table
            .inner_join(stream_followers::table.on(stream_followers::stream_id.eq(streams::id)))
            .filter(stream_followers::user_id.eq(user.id))
            .filter(stream_followers::added_to_navbar.eq(true))
            .select(streams::all_columns)
            .order(stream_followers::navbar_position.asc())
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Stream::from).collect())
    }

    /// Discover public streams (BUG-023 fix: batch load instead of N+1 loop)
    async fn discover_streams(
        &self,
        ctx: &Context<'_>,
        sort_by: Option<StreamSortType>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Stream>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let sort_by = sort_by.unwrap_or(StreamSortType::Popular);
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let mut query = streams::table
            .filter(streams::is_public.eq(true))
            .filter(streams::is_discoverable.eq(true))
            .into_boxed();

        query = match sort_by {
            StreamSortType::New => query.order(streams::created_at.desc()),
            StreamSortType::Popular | StreamSortType::Trending => {
                query.order(streams::updated_at.desc())
            }
        };

        let results: Vec<DbStream> = query
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Stream::from).collect())
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
        let conn = &mut get_conn(pool).await?;

        if query.trim().is_empty() {
            return Err(
                TinyBoardsError::from_message(400, "Search query cannot be empty").into(),
            );
        }

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);
        let pattern = format!("%{}%", query);

        let results: Vec<DbStream> = streams::table
            .filter(streams::is_public.eq(true))
            .filter(
                streams::name
                    .ilike(&pattern)
                    .or(streams::description.ilike(&pattern)),
            )
            .order(streams::updated_at.desc())
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Stream::from).collect())
    }
}
