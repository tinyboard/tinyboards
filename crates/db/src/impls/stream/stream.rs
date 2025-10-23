use crate::{
    models::stream::stream::{CreateStreamForm, Stream, UpdateStreamForm},
    schema::streams,
    traits::Crud,
    utils::{get_conn, naive_now, DbPool},
};
use diesel::{dsl::*, prelude::*, result::Error};
use diesel_async::RunQueryDsl;
use rand::Rng;

/// Maximum number of streams a user can create
pub const MAX_STREAMS_PER_USER: i64 = 50;

impl Stream {
    /// Check if a user has reached their stream creation quota
    pub async fn check_user_quota(pool: &DbPool, user_creator_id: i32) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::streams::dsl::*;

        let count = streams
            .filter(creator_id.eq(user_creator_id))
            .count()
            .get_result::<i64>(conn)
            .await?;

        Ok(count < MAX_STREAMS_PER_USER)
    }

    /// Get stream count for a user
    pub async fn get_user_stream_count(pool: &DbPool, user_id: i32) -> Result<i64, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::streams::dsl::*;

        streams
            .filter(creator_id.eq(user_id))
            .count()
            .get_result::<i64>(conn)
            .await
    }

    /// Check if a slug is unique for a given user
    pub async fn is_slug_unique(
        pool: &DbPool,
        user_id: i32,
        stream_slug: &str,
        exclude_stream_id: Option<i32>,
    ) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::streams::dsl::*;

        let mut query = streams
            .filter(creator_id.eq(user_id))
            .filter(slug.eq(stream_slug))
            .into_boxed();

        if let Some(exclude_id) = exclude_stream_id {
            query = query.filter(id.ne(exclude_id));
        }

        let count = query.count().get_result::<i64>(conn).await?;
        Ok(count == 0)
    }

    /// Generate a unique slug from a stream name
    pub fn generate_slug(name: &str) -> String {
        name.to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }

    /// Generate a cryptographically secure share token
    pub fn generate_share_token() -> String {
        use rand::distributions::Alphanumeric;

        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        token
    }

    /// Get stream by slug for a specific user
    pub async fn get_by_slug(
        pool: &DbPool,
        user_id: i32,
        stream_slug: &str,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::streams::dsl::*;

        streams
            .filter(creator_id.eq(user_id))
            .filter(slug.eq(stream_slug))
            .first::<Self>(conn)
            .await
    }

    /// Get stream by share token
    pub async fn get_by_share_token(pool: &DbPool, token: &str) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::streams::dsl::*;

        streams
            .filter(share_token.eq(Some(token)))
            .first::<Self>(conn)
            .await
    }

    /// Get all streams for a user (creator)
    pub async fn get_user_streams(
        pool: &DbPool,
        user_id: i32,
        _include_deleted: bool,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::streams::dsl::*;

        streams
            .filter(creator_id.eq(user_id))
            .order_by(created_at.desc())
            .load::<Self>(conn)
            .await
    }

    /// Get all public streams (for discovery)
    pub async fn get_public_streams(
        pool: &DbPool,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::streams::dsl::*;

        let mut query = streams
            .filter(is_public.eq(true))
            .filter(is_discoverable.eq(true))
            .order_by(created_at.desc())
            .into_boxed();

        if let Some(lim) = limit {
            query = query.limit(lim);
        }

        if let Some(off) = offset {
            query = query.offset(off);
        }

        query.load::<Self>(conn).await
    }


    /// Update stream visibility
    pub async fn update_visibility(
        pool: &DbPool,
        stream_id: i32,
        public: bool,
        token: Option<String>,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::streams::dsl::*;

        diesel::update(streams.find(stream_id))
            .set((
                is_public.eq(public),
                share_token.eq(token),
                updated_at.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    /// Regenerate share token for a stream
    pub async fn regenerate_share_token(pool: &DbPool, stream_id: i32) -> Result<Self, Error> {
        let new_token = Self::generate_share_token();
        let conn = &mut get_conn(pool).await?;
        use crate::schema::streams::dsl::*;

        diesel::update(streams.find(stream_id))
            .set((share_token.eq(Some(new_token)), updated_at.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    /// Check if a user owns a stream
    pub async fn user_owns_stream(
        pool: &DbPool,
        stream_id: i32,
        user_id: i32,
    ) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::streams::dsl::*;

        let count = streams
            .filter(id.eq(stream_id))
            .filter(creator_id.eq(user_id))
            .count()
            .get_result::<i64>(conn)
            .await?;

        Ok(count > 0)
    }

    /// Check if a user can access a stream (owner, public, or has share link)
    pub async fn user_can_access(
        pool: &DbPool,
        stream_id: i32,
        user_id: Option<i32>,
        token: Option<&str>,
    ) -> Result<bool, Error> {
        let stream = Self::read(pool, stream_id).await?;

        // Owner always has access
        if let Some(uid) = user_id {
            if stream.creator_id == uid {
                return Ok(true);
            }
        }

        // Public streams are accessible to everyone
        if stream.is_public {
            return Ok(true);
        }

        // Check share token
        if let (Some(stream_token), Some(provided_token)) = (&stream.share_token, token) {
            return Ok(stream_token == provided_token);
        }

        // Check if user is a follower (requires separate query)
        if let Some(uid) = user_id {
            use crate::schema::stream_followers;
            let conn = &mut get_conn(pool).await?;

            let follower_count = stream_followers::table
                .filter(stream_followers::stream_id.eq(stream_id))
                .filter(stream_followers::user_id.eq(uid))
                .count()
                .get_result::<i64>(conn)
                .await?;

            return Ok(follower_count > 0);
        }

        Ok(false)
    }

    /// Get stream by slug and username
    pub async fn get_by_slug_and_username(
        pool: &DbPool,
        stream_slug: &str,
        username: &str,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::{streams, users};

        streams::table
            .inner_join(users::table.on(streams::creator_id.eq(users::id)))
            .filter(streams::slug.eq(stream_slug))
            .filter(users::name.eq(username))
            .select(streams::all_columns)
            .first::<Self>(conn)
            .await
    }

    /// List streams for a user with pagination
    pub async fn list_for_user(
        pool: &DbPool,
        user_id: i32,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::streams::dsl::*;

        let mut query = streams
            .filter(creator_id.eq(user_id))
            .order_by(created_at.desc())
            .into_boxed();

        if let Some(lim) = limit {
            query = query.limit(lim);
        }

        if let Some(off) = offset {
            query = query.offset(off);
        }

        query.load::<Self>(conn).await
    }

    /// List public streams with sort and pagination
    pub async fn list_public(
        pool: &DbPool,
        sort_by: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::streams::dsl::*;

        let mut query = streams
            .filter(is_public.eq(true))
            .into_boxed();

        // Apply sorting
        query = match sort_by.as_deref() {
            Some("old") => query.order_by(created_at.asc()),
            Some("new") | None | _ => query.order_by(created_at.desc()),
        };

        if let Some(lim) = limit {
            query = query.limit(lim);
        }

        if let Some(off) = offset {
            query = query.offset(off);
        }

        query.load::<Self>(conn).await
    }

    /// Search streams by name or description
    pub async fn search(
        pool: &DbPool,
        query_text: &str,
        user_id: Option<i32>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::streams::dsl::*;

        let search_pattern = format!("%{}%", query_text);

        let mut query = streams
            .filter(
                name.ilike(&search_pattern)
                    .or(description.ilike(&search_pattern))
            )
            .into_boxed();

        // If user is not provided, only show public streams
        if user_id.is_none() {
            query = query.filter(is_public.eq(true));
        } else if let Some(uid) = user_id {
            // Show public streams or streams owned by the user
            query = query.filter(
                is_public.eq(true)
                    .or(creator_id.eq(uid))
            );
        }

        query = query.order_by(created_at.desc());

        if let Some(lim) = limit {
            query = query.limit(lim);
        }

        if let Some(off) = offset {
            query = query.offset(off);
        }

        query.load::<Self>(conn).await
    }

    /// Update last viewed timestamp for a stream
    pub async fn update_last_viewed(
        pool: &DbPool,
        stream_id: i32,
        _user_id: i32,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::streams::dsl::*;

        diesel::update(streams.find(stream_id))
            .set(last_viewed_at.eq(Some(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    /// Update share token
    pub async fn update_share_token(
        pool: &DbPool,
        stream_id: i32,
        token: Option<&str>,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::streams::dsl::*;

        diesel::update(streams.find(stream_id))
            .set((
                share_token.eq(token),
                updated_at.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    /// Check if slug exists for user
    pub async fn slug_exists_for_user(
        pool: &DbPool,
        user_id: i32,
        stream_slug: &str,
    ) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::streams::dsl::*;

        let count = streams
            .filter(creator_id.eq(user_id))
            .filter(slug.eq(stream_slug))
            .count()
            .get_result::<i64>(conn)
            .await?;

        Ok(count > 0)
    }
}

#[async_trait::async_trait]
impl Crud for Stream {
    type Form = CreateStreamForm;
    type IdType = i32;

    async fn read(pool: &DbPool, stream_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        streams::table.find(stream_id).first::<Self>(conn).await
    }

    async fn delete(pool: &DbPool, stream_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(streams::table.find(stream_id))
            .execute(conn)
            .await
    }

    async fn create(pool: &DbPool, form: &CreateStreamForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::insert_into(streams::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(pool: &DbPool, stream_id: i32, form: &CreateStreamForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::update(streams::table.find(stream_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}
