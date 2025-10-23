use crate::{
    models::stream::stream_follower::{
        NavbarStreamConfig, StreamFollower, StreamFollowerForm, UpdateStreamFollowerForm,
    },
    schema::{stream_followers, streams},
    traits::Joinable,
    utils::{get_conn, DbPool},
};
use diesel::{dsl::*, prelude::*, result::Error};
use diesel_async::RunQueryDsl;

impl StreamFollower {
    /// Follow a stream
    pub async fn follow(pool: &DbPool, form: &StreamFollowerForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::insert_into(stream_followers::table)
            .values(form)
            .on_conflict((stream_followers::stream_id, stream_followers::user_id))
            .do_nothing()
            .get_result::<Self>(conn)
            .await
    }

    /// Unfollow a stream
    pub async fn unfollow(pool: &DbPool, stream_id: i32, user_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_followers::dsl;

        diesel::delete(
            stream_followers::table
                .filter(dsl::stream_id.eq(stream_id))
                .filter(dsl::user_id.eq(user_id)),
        )
        .execute(conn)
        .await
    }

    /// Check if a user is following a stream
    pub async fn is_following(
        pool: &DbPool,
        stream_id: i32,
        user_id: i32,
    ) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_followers::dsl;

        let count = stream_followers::table
            .filter(dsl::stream_id.eq(stream_id))
            .filter(dsl::user_id.eq(user_id))
            .count()
            .get_result::<i64>(conn)
            .await?;

        Ok(count > 0)
    }

    /// Get all streams a user is following
    pub async fn get_followed_streams(pool: &DbPool, user_id: i32) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_followers::dsl;

        stream_followers::table
            .filter(dsl::user_id.eq(user_id))
            .order_by(dsl::creation_date.desc())
            .load::<Self>(conn)
            .await
    }

    /// Get all followers of a stream
    pub async fn get_stream_followers(pool: &DbPool, stream_id: i32) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_followers::dsl;

        stream_followers::table
            .filter(dsl::stream_id.eq(stream_id))
            .order_by(dsl::creation_date.desc())
            .load::<Self>(conn)
            .await
    }

    /// Get follower count for a stream
    pub async fn get_follower_count(pool: &DbPool, stream_id: i32) -> Result<i64, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_followers::dsl;

        stream_followers::table
            .filter(dsl::stream_id.eq(stream_id))
            .count()
            .get_result::<i64>(conn)
            .await
    }

    /// Update navbar settings for a followed stream
    pub async fn update_navbar_settings(
        pool: &DbPool,
        stream_id: i32,
        user_id: i32,
        form: &UpdateStreamFollowerForm,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_followers::dsl;

        diesel::update(
            stream_followers::table
                .filter(dsl::stream_id.eq(stream_id))
                .filter(dsl::user_id.eq(user_id)),
        )
        .set(form)
        .get_result::<Self>(conn)
        .await
    }

    /// Add stream to user's navbar
    pub async fn add_to_navbar(
        pool: &DbPool,
        stream_id: i32,
        user_id: i32,
        position: Option<i32>,
    ) -> Result<Self, Error> {
        let form = UpdateStreamFollowerForm {
            added_to_navbar: Some(true),
            navbar_position: Some(position),
        };

        Self::update_navbar_settings(pool, stream_id, user_id, &form).await
    }

    /// Remove stream from user's navbar
    pub async fn remove_from_navbar(
        pool: &DbPool,
        stream_id: i32,
        user_id: i32,
    ) -> Result<Self, Error> {
        let form = UpdateStreamFollowerForm {
            added_to_navbar: Some(false),
            navbar_position: Some(None),
        };

        Self::update_navbar_settings(pool, stream_id, user_id, &form).await
    }

    /// Get all streams in a user's navbar, ordered by position
    pub async fn get_navbar_streams(
        pool: &DbPool,
        user_id: i32,
    ) -> Result<Vec<NavbarStreamConfig>, Error> {
        let conn = &mut get_conn(pool).await?;

        let result = stream_followers::table
            .inner_join(streams::table)
            .filter(stream_followers::user_id.eq(user_id))
            .filter(stream_followers::added_to_navbar.eq(true))
            .select((
                streams::id,
                streams::name,
                streams::slug,
                stream_followers::navbar_position,
            ))
            .order_by(stream_followers::navbar_position.asc())
            .load::<(i32, String, String, Option<i32>)>(conn)
            .await?
            .into_iter()
            .map(|(id, name, slug, pos)| NavbarStreamConfig {
                stream_id: id,
                stream_name: name,
                stream_slug: slug,
                position: pos.unwrap_or(999),
            })
            .collect::<Vec<_>>();

        Ok(result)
    }

    /// Reorder navbar streams for a user
    pub async fn reorder_navbar_streams(
        pool: &DbPool,
        user_id: i32,
        stream_positions: Vec<(i32, i32)>, // Vec<(stream_id, position)>
    ) -> Result<(), Error> {
        let conn = &mut get_conn(pool).await?;

        for (stream_id, position) in stream_positions {
            diesel::update(
                stream_followers::table
                    .filter(stream_followers::user_id.eq(user_id))
                    .filter(stream_followers::stream_id.eq(stream_id)),
            )
            .set(stream_followers::navbar_position.eq(Some(position)))
            .execute(conn)
            .await?;
        }

        Ok(())
    }

    /// Get a specific follower relationship
    pub async fn get_follower(
        pool: &DbPool,
        stream_id: i32,
        user_id: i32,
    ) -> Result<Option<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_followers::dsl;

        stream_followers::table
            .filter(dsl::stream_id.eq(stream_id))
            .filter(dsl::user_id.eq(user_id))
            .first::<Self>(conn)
            .await
            .optional()
    }

    /// Get count of streams a user is following
    pub async fn get_user_following_count(pool: &DbPool, user_id: i32) -> Result<i64, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_followers::dsl;

        stream_followers::table
            .filter(dsl::user_id.eq(user_id))
            .count()
            .get_result::<i64>(conn)
            .await
    }

    /// List followed stream IDs with pagination
    pub async fn list_followed_stream_ids(
        pool: &DbPool,
        user_id: i32,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<i32>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_followers::dsl;

        let mut query = stream_followers::table
            .filter(dsl::user_id.eq(user_id))
            .select(dsl::stream_id)
            .order_by(dsl::creation_date.desc())
            .into_boxed();

        if let Some(lim) = limit {
            query = query.limit(lim);
        }

        if let Some(off) = offset {
            query = query.offset(off);
        }

        query.load::<i32>(conn).await
    }

    /// List navbar streams for a user
    pub async fn list_navbar_streams(
        pool: &DbPool,
        user_id: i32,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_followers::dsl;

        stream_followers::table
            .filter(dsl::user_id.eq(user_id))
            .filter(dsl::added_to_navbar.eq(true))
            .order_by(dsl::navbar_position.asc())
            .load::<Self>(conn)
            .await
    }

    /// List followers for a stream with pagination
    pub async fn list_for_stream(
        pool: &DbPool,
        stream_id: i32,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_followers::dsl;

        let mut query = stream_followers::table
            .filter(dsl::stream_id.eq(stream_id))
            .order_by(dsl::creation_date.desc())
            .into_boxed();

        if let Some(lim) = limit {
            query = query.limit(lim);
        }

        if let Some(off) = offset {
            query = query.offset(off);
        }

        query.load::<Self>(conn).await
    }

    /// Get a specific follower relationship
    pub async fn get(
        pool: &DbPool,
        user_id: i32,
        stream_id: i32,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_followers::dsl;

        stream_followers::table
            .filter(dsl::stream_id.eq(stream_id))
            .filter(dsl::user_id.eq(user_id))
            .first::<Self>(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Joinable for StreamFollower {
    type Form = StreamFollowerForm;

    async fn join(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        Self::follow(pool, form).await
    }

    async fn leave(pool: &DbPool, form: &Self::Form) -> Result<usize, Error> {
        Self::unfollow(pool, form.stream_id, form.user_id).await
    }
}
