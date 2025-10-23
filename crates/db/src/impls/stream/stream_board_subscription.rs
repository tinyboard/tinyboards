use crate::{
    models::stream::stream_board_subscription::{
        StreamBoardSubscription, StreamBoardSubscriptionForm,
    },
    schema::stream_board_subscriptions,
    utils::{get_conn, DbPool},
};
use diesel::{dsl::*, prelude::*, result::Error, upsert::excluded};
use diesel_async::RunQueryDsl;

impl StreamBoardSubscription {
    /// Add a board subscription to a stream
    pub async fn add_subscription(
        pool: &DbPool,
        form: &StreamBoardSubscriptionForm,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::insert_into(stream_board_subscriptions::table)
            .values(form)
            .on_conflict((
                stream_board_subscriptions::stream_id,
                stream_board_subscriptions::board_id,
            ))
            .do_update()
            .set(stream_board_subscriptions::include_all_posts.eq(form.include_all_posts))
            .get_result::<Self>(conn)
            .await
    }

    /// Remove a board subscription from a stream
    pub async fn remove_subscription(
        pool: &DbPool,
        stream_id: i32,
        board_id: i32,
    ) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_board_subscriptions::dsl;

        diesel::delete(
            stream_board_subscriptions::table
                .filter(dsl::stream_id.eq(stream_id))
                .filter(dsl::board_id.eq(board_id)),
        )
        .execute(conn)
        .await
    }

    /// Remove all board subscriptions for a stream
    pub async fn remove_all_subscriptions(pool: &DbPool, stream_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_board_subscriptions::dsl;

        diesel::delete(
            stream_board_subscriptions::table.filter(dsl::stream_id.eq(stream_id)),
        )
        .execute(conn)
        .await
    }

    /// Get all board subscriptions for a stream
    pub async fn get_stream_subscriptions(
        pool: &DbPool,
        stream_id: i32,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_board_subscriptions::dsl;

        stream_board_subscriptions::table
            .filter(dsl::stream_id.eq(stream_id))
            .order_by(dsl::creation_date.desc())
            .load::<Self>(conn)
            .await
    }

    /// Get all boards subscribed to by a stream
    pub async fn get_subscribed_board_ids(
        pool: &DbPool,
        stream_id: i32,
    ) -> Result<Vec<i32>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_board_subscriptions::dsl;

        stream_board_subscriptions::table
            .filter(dsl::stream_id.eq(stream_id))
            .select(dsl::board_id)
            .load::<i32>(conn)
            .await
    }

    /// Check if a stream is subscribed to a board
    pub async fn is_subscribed(
        pool: &DbPool,
        stream_id: i32,
        board_id: i32,
    ) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_board_subscriptions::dsl;

        let count = stream_board_subscriptions::table
            .filter(dsl::stream_id.eq(stream_id))
            .filter(dsl::board_id.eq(board_id))
            .count()
            .get_result::<i64>(conn)
            .await?;

        Ok(count > 0)
    }

    /// Get subscription count for a stream
    pub async fn get_subscription_count(pool: &DbPool, stream_id: i32) -> Result<i64, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_board_subscriptions::dsl;

        stream_board_subscriptions::table
            .filter(dsl::stream_id.eq(stream_id))
            .count()
            .get_result::<i64>(conn)
            .await
    }

    /// Batch add multiple board subscriptions
    pub async fn batch_add_subscriptions(
        pool: &DbPool,
        forms: &[StreamBoardSubscriptionForm],
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::insert_into(stream_board_subscriptions::table)
            .values(forms)
            .on_conflict((
                stream_board_subscriptions::stream_id,
                stream_board_subscriptions::board_id,
            ))
            .do_update()
            .set(stream_board_subscriptions::include_all_posts.eq(excluded(stream_board_subscriptions::include_all_posts)))
            .get_results::<Self>(conn)
            .await
    }

    /// Update include_all_posts setting for a subscription
    pub async fn update_include_all_posts(
        pool: &DbPool,
        stream_id: i32,
        board_id: i32,
        include_all: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_board_subscriptions::dsl;

        diesel::update(
            stream_board_subscriptions::table
                .filter(dsl::stream_id.eq(stream_id))
                .filter(dsl::board_id.eq(board_id)),
        )
        .set(dsl::include_all_posts.eq(include_all))
        .get_result::<Self>(conn)
        .await
    }

    /// Get a specific subscription
    pub async fn get_subscription(
        pool: &DbPool,
        stream_id: i32,
        board_id: i32,
    ) -> Result<Option<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_board_subscriptions::dsl;

        stream_board_subscriptions::table
            .filter(dsl::stream_id.eq(stream_id))
            .filter(dsl::board_id.eq(board_id))
            .first::<Self>(conn)
            .await
            .optional()
    }

    /// Get all boards subscribed with "include all posts" enabled
    pub async fn get_full_board_subscription_ids(
        pool: &DbPool,
        stream_id: i32,
    ) -> Result<Vec<i32>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_board_subscriptions::dsl;

        stream_board_subscriptions::table
            .filter(dsl::stream_id.eq(stream_id))
            .filter(dsl::include_all_posts.eq(true))
            .select(dsl::board_id)
            .load::<i32>(conn)
            .await
    }

    /// List board subscriptions for a stream
    pub async fn list_for_stream(
        pool: &DbPool,
        stream_id: i32,
    ) -> Result<Vec<Self>, Error> {
        Self::get_stream_subscriptions(pool, stream_id).await
    }

    /// Check if a subscription exists
    pub async fn exists(
        pool: &DbPool,
        stream_id: i32,
        board_id: i32,
    ) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_board_subscriptions::dsl;

        let count = stream_board_subscriptions::table
            .filter(dsl::stream_id.eq(stream_id))
            .filter(dsl::board_id.eq(board_id))
            .count()
            .get_result::<i64>(conn)
            .await?;

        Ok(count > 0)
    }

    /// Create a new subscription
    pub async fn create(
        pool: &DbPool,
        form: &StreamBoardSubscriptionForm,
    ) -> Result<Self, Error> {
        Self::add_subscription(pool, form).await
    }

    /// Delete a subscription
    pub async fn delete(
        pool: &DbPool,
        stream_id: i32,
        board_id: i32,
    ) -> Result<usize, Error> {
        Self::remove_subscription(pool, stream_id, board_id).await
    }

    /// Delete all subscriptions for a stream
    pub async fn delete_all_for_stream(
        pool: &DbPool,
        stream_id: i32,
    ) -> Result<usize, Error> {
        Self::remove_all_subscriptions(pool, stream_id).await
    }
}
