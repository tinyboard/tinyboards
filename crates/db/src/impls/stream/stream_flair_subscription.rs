use crate::{
    models::stream::stream_flair_subscription::{
        StreamFlairSubscription, StreamFlairSubscriptionForm,
    },
    schema::stream_flair_subscriptions,
    utils::{get_conn, DbPool},
};
use diesel::{dsl::*, prelude::*, result::Error};
use diesel_async::RunQueryDsl;

impl StreamFlairSubscription {
    /// Add a flair subscription to a stream
    pub async fn add_subscription(
        pool: &DbPool,
        form: &StreamFlairSubscriptionForm,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::insert_into(stream_flair_subscriptions::table)
            .values(form)
            .on_conflict((
                stream_flair_subscriptions::stream_id,
                stream_flair_subscriptions::board_id,
                stream_flair_subscriptions::flair_id,
            ))
            .do_nothing()
            .get_result::<Self>(conn)
            .await
    }

    /// Remove a specific flair subscription from a stream
    pub async fn remove_subscription(
        pool: &DbPool,
        stream_id: i32,
        board_id: i32,
        flair_id: i32,
    ) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_flair_subscriptions::dsl;

        diesel::delete(
            stream_flair_subscriptions::table
                .filter(dsl::stream_id.eq(stream_id))
                .filter(dsl::board_id.eq(board_id))
                .filter(dsl::flair_id.eq(flair_id)),
        )
        .execute(conn)
        .await
    }

    /// Remove all flair subscriptions for a stream from a specific board
    pub async fn remove_board_subscriptions(
        pool: &DbPool,
        stream_id: i32,
        board_id: i32,
    ) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_flair_subscriptions::dsl;

        diesel::delete(
            stream_flair_subscriptions::table
                .filter(dsl::stream_id.eq(stream_id))
                .filter(dsl::board_id.eq(board_id)),
        )
        .execute(conn)
        .await
    }

    /// Remove all flair subscriptions for a stream
    pub async fn remove_all_subscriptions(pool: &DbPool, stream_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_flair_subscriptions::dsl;

        diesel::delete(
            stream_flair_subscriptions::table.filter(dsl::stream_id.eq(stream_id)),
        )
        .execute(conn)
        .await
    }

    /// Get all flair subscriptions for a stream
    pub async fn get_stream_subscriptions(
        pool: &DbPool,
        stream_id: i32,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_flair_subscriptions::dsl;

        stream_flair_subscriptions::table
            .filter(dsl::stream_id.eq(stream_id))
            .order_by(dsl::creation_date.desc())
            .load::<Self>(conn)
            .await
    }

    /// Get all flair subscriptions for a stream from a specific board
    pub async fn get_board_subscriptions(
        pool: &DbPool,
        stream_id: i32,
        board_id: i32,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_flair_subscriptions::dsl;

        stream_flair_subscriptions::table
            .filter(dsl::stream_id.eq(stream_id))
            .filter(dsl::board_id.eq(board_id))
            .order_by(dsl::creation_date.desc())
            .load::<Self>(conn)
            .await
    }

    /// Check if a stream is subscribed to a specific flair
    pub async fn is_subscribed(
        pool: &DbPool,
        stream_id: i32,
        board_id: i32,
        flair_id: i32,
    ) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_flair_subscriptions::dsl;

        let count = stream_flair_subscriptions::table
            .filter(dsl::stream_id.eq(stream_id))
            .filter(dsl::board_id.eq(board_id))
            .filter(dsl::flair_id.eq(flair_id))
            .count()
            .get_result::<i64>(conn)
            .await?;

        Ok(count > 0)
    }

    /// Get count of flair subscriptions for a stream
    pub async fn get_subscription_count(pool: &DbPool, stream_id: i32) -> Result<i64, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_flair_subscriptions::dsl;

        stream_flair_subscriptions::table
            .filter(dsl::stream_id.eq(stream_id))
            .count()
            .get_result::<i64>(conn)
            .await
    }

    /// Batch add multiple flair subscriptions
    pub async fn batch_add_subscriptions(
        pool: &DbPool,
        forms: &[StreamFlairSubscriptionForm],
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::insert_into(stream_flair_subscriptions::table)
            .values(forms)
            .on_conflict((
                stream_flair_subscriptions::stream_id,
                stream_flair_subscriptions::board_id,
                stream_flair_subscriptions::flair_id,
            ))
            .do_nothing()
            .get_results::<Self>(conn)
            .await
    }

    /// Get all boards with flair subscriptions for a stream
    pub async fn get_subscribed_boards(
        pool: &DbPool,
        stream_id: i32,
    ) -> Result<Vec<i32>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_flair_subscriptions::dsl;

        stream_flair_subscriptions::table
            .filter(dsl::stream_id.eq(stream_id))
            .select(dsl::board_id)
            .distinct()
            .load::<i32>(conn)
            .await
    }

    /// List flair subscriptions for a stream
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
        flair_id: i32,
    ) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_flair_subscriptions::dsl;

        let count = stream_flair_subscriptions::table
            .filter(dsl::stream_id.eq(stream_id))
            .filter(dsl::flair_id.eq(flair_id))
            .count()
            .get_result::<i64>(conn)
            .await?;

        Ok(count > 0)
    }

    /// Create a new subscription
    pub async fn create(
        pool: &DbPool,
        form: &StreamFlairSubscriptionForm,
    ) -> Result<Self, Error> {
        Self::add_subscription(pool, form).await
    }

    /// Delete a subscription
    pub async fn delete(
        pool: &DbPool,
        stream_id: i32,
        flair_id: i32,
    ) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stream_flair_subscriptions::dsl;

        diesel::delete(
            stream_flair_subscriptions::table
                .filter(dsl::stream_id.eq(stream_id))
                .filter(dsl::flair_id.eq(flair_id)),
        )
        .execute(conn)
        .await
    }

    /// Delete all subscriptions for a stream
    pub async fn delete_all_for_stream(
        pool: &DbPool,
        stream_id: i32,
    ) -> Result<usize, Error> {
        Self::remove_all_subscriptions(pool, stream_id).await
    }
}
