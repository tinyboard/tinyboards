use crate::{
    aggregates::structs::StreamAggregates,
    schema::{stream_aggregates, stream_board_subscriptions, stream_flair_subscriptions, stream_followers},
    utils::{get_conn, naive_now, DbPool},
};
use diesel::{dsl::*, prelude::*, result::Error};
use diesel_async::RunQueryDsl;

impl StreamAggregates {
    /// Create initial aggregates for a new stream
    pub async fn create_for_stream(pool: &DbPool, stream_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::insert_into(stream_aggregates::table)
            .values((
                stream_aggregates::stream_id.eq(stream_id),
                stream_aggregates::follower_count.eq(0i32),
                stream_aggregates::board_subscription_count.eq(0i32),
                stream_aggregates::flair_subscription_count.eq(0i32),
                stream_aggregates::total_subscription_count.eq(0i32),
                stream_aggregates::updated.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    /// Get aggregates for a stream
    pub async fn get_for_stream(pool: &DbPool, stream_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        stream_aggregates::table
            .filter(stream_aggregates::stream_id.eq(stream_id))
            .first::<Self>(conn)
            .await
    }

    /// Read aggregates for a stream (alias for get_for_stream)
    pub async fn read(pool: &DbPool, stream_id: i32) -> Result<Self, Error> {
        Self::get_for_stream(pool, stream_id).await
    }

    /// Recalculate and update all aggregates for a stream
    pub async fn refresh(pool: &DbPool, stream_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        // Get follower count
        let follower_count = stream_followers::table
            .filter(stream_followers::stream_id.eq(stream_id))
            .count()
            .get_result::<i64>(conn)
            .await? as i32;

        // Get board subscription count
        let board_count = stream_board_subscriptions::table
            .filter(stream_board_subscriptions::stream_id.eq(stream_id))
            .count()
            .get_result::<i64>(conn)
            .await? as i32;

        // Get flair subscription count
        let flair_count = stream_flair_subscriptions::table
            .filter(stream_flair_subscriptions::stream_id.eq(stream_id))
            .count()
            .get_result::<i64>(conn)
            .await? as i32;

        // Total is board + flair subscriptions
        let total_count = board_count + flair_count;

        // Update aggregates
        diesel::update(stream_aggregates::table.filter(stream_aggregates::stream_id.eq(stream_id)))
            .set((
                stream_aggregates::follower_count.eq(follower_count),
                stream_aggregates::board_subscription_count.eq(board_count),
                stream_aggregates::flair_subscription_count.eq(flair_count),
                stream_aggregates::total_subscription_count.eq(total_count),
                stream_aggregates::updated.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    /// Increment follower count
    pub async fn increment_followers(pool: &DbPool, stream_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::update(stream_aggregates::table.filter(stream_aggregates::stream_id.eq(stream_id)))
            .set((
                stream_aggregates::follower_count.eq(stream_aggregates::follower_count + 1),
                stream_aggregates::updated.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    /// Decrement follower count
    pub async fn decrement_followers(pool: &DbPool, stream_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::update(stream_aggregates::table.filter(stream_aggregates::stream_id.eq(stream_id)))
            .set((
                stream_aggregates::follower_count.eq(stream_aggregates::follower_count - 1),
                stream_aggregates::updated.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    /// Increment board subscription count
    pub async fn increment_board_subscriptions(
        pool: &DbPool,
        stream_id: i32,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::update(stream_aggregates::table.filter(stream_aggregates::stream_id.eq(stream_id)))
            .set((
                stream_aggregates::board_subscription_count
                    .eq(stream_aggregates::board_subscription_count + 1),
                stream_aggregates::total_subscription_count
                    .eq(stream_aggregates::total_subscription_count + 1),
                stream_aggregates::updated.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    /// Decrement board subscription count
    pub async fn decrement_board_subscriptions(
        pool: &DbPool,
        stream_id: i32,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::update(stream_aggregates::table.filter(stream_aggregates::stream_id.eq(stream_id)))
            .set((
                stream_aggregates::board_subscription_count
                    .eq(stream_aggregates::board_subscription_count - 1),
                stream_aggregates::total_subscription_count
                    .eq(stream_aggregates::total_subscription_count - 1),
                stream_aggregates::updated.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    /// Increment flair subscription count
    pub async fn increment_flair_subscriptions(
        pool: &DbPool,
        stream_id: i32,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::update(stream_aggregates::table.filter(stream_aggregates::stream_id.eq(stream_id)))
            .set((
                stream_aggregates::flair_subscription_count
                    .eq(stream_aggregates::flair_subscription_count + 1),
                stream_aggregates::total_subscription_count
                    .eq(stream_aggregates::total_subscription_count + 1),
                stream_aggregates::updated.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    /// Decrement flair subscription count
    pub async fn decrement_flair_subscriptions(
        pool: &DbPool,
        stream_id: i32,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::update(stream_aggregates::table.filter(stream_aggregates::stream_id.eq(stream_id)))
            .set((
                stream_aggregates::flair_subscription_count
                    .eq(stream_aggregates::flair_subscription_count - 1),
                stream_aggregates::total_subscription_count
                    .eq(stream_aggregates::total_subscription_count - 1),
                stream_aggregates::updated.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    /// Delete aggregates for a stream
    pub async fn delete_for_stream(pool: &DbPool, stream_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::delete(stream_aggregates::table.filter(stream_aggregates::stream_id.eq(stream_id)))
            .execute(conn)
            .await
    }

    /// Get aggregates for multiple streams
    pub async fn get_for_streams(
        pool: &DbPool,
        stream_ids: Vec<i32>,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;

        stream_aggregates::table
            .filter(stream_aggregates::stream_id.eq_any(stream_ids))
            .load::<Self>(conn)
            .await
    }
}
