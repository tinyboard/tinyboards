use crate::{
    aggregates::structs::PostAggregates,
    models::{
        post::posts::Post,
        stream::stream::Stream,
    },
    schema::{post_aggregates, posts, stream_board_subscriptions, stream_flair_subscriptions},
    utils::{get_conn, limit_and_offset, DbPool},
    SortType,
};
use chrono::{NaiveDateTime, Utc};
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;


/// Parameters for generating a stream feed
#[derive(Debug, Clone)]
pub struct StreamFeedParams {
    pub stream_id: i32,
    pub sort: SortType,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub nsfw: bool,
    pub exclude_deleted: bool,
    pub exclude_removed: bool,
}

impl Default for StreamFeedParams {
    fn default() -> Self {
        Self {
            stream_id: 0,
            sort: SortType::Hot,
            page: Some(1),
            limit: Some(25),
            nsfw: false,
            exclude_deleted: true,
            exclude_removed: true,
        }
    }
}

/// Feed generator for streams
pub struct StreamFeedGenerator;

impl StreamFeedGenerator {
    /// Generate a feed for a stream combining both board and flair subscriptions
    ///
    /// The feed includes:
    /// 1. All posts from boards with `include_all_posts = true`
    /// 2. Posts with specific flairs from flair subscriptions
    ///
    /// Posts are deduplicated and sorted according to the sort type
    pub async fn generate_feed(
        pool: &DbPool,
        params: &StreamFeedParams,
    ) -> Result<Vec<(Post, PostAggregates)>, Error> {
        let conn = &mut get_conn(pool).await?;

        // Get board IDs where stream wants all posts
        let full_board_ids: Vec<i32> = stream_board_subscriptions::table
            .filter(stream_board_subscriptions::stream_id.eq(params.stream_id))
            .filter(stream_board_subscriptions::include_all_posts.eq(true))
            .select(stream_board_subscriptions::board_id)
            .load::<i32>(conn)
            .await?;

        // Get flair subscriptions (board_id, flair_id pairs)
        let flair_subs: Vec<(i32, i32)> = stream_flair_subscriptions::table
            .filter(stream_flair_subscriptions::stream_id.eq(params.stream_id))
            .select((
                stream_flair_subscriptions::board_id,
                stream_flair_subscriptions::flair_id,
            ))
            .load::<(i32, i32)>(conn)
            .await?;

        // Build base query
        let mut query = posts::table
            .inner_join(post_aggregates::table)
            .select((posts::all_columns, post_aggregates::all_columns))
            .into_boxed();

        // Apply subscription filters
        // The query logic is: (board_id IN full_boards) OR (board_id, flair_id) IN flair_subs
        if !full_board_ids.is_empty() {
            // Filter by board IDs where stream wants all posts
            query = query.filter(posts::board_id.eq_any(&full_board_ids));
        } else if !flair_subs.is_empty() {
            // For flair subscriptions, we need to match both board_id and flair_id
            // Note: This assumes there's a flair_id column in posts table
            // If flairs are stored differently, this logic needs adjustment

            // Extract just the board IDs for now (until flair_id is added to posts table)
            let flair_board_ids: Vec<i32> = flair_subs.iter().map(|(board_id, _)| *board_id).collect();
            query = query.filter(posts::board_id.eq_any(flair_board_ids));

            // TODO: When flair_id is added to posts table, use proper filtering:
            // let mut flair_conditions = Vec::new();
            // for (board_id, flair_id) in flair_subs.iter() {
            //     flair_conditions.push(posts::board_id.eq(board_id).and(posts::flair_id.eq(flair_id)));
            // }
            // query = query.filter(diesel::dsl::any(flair_conditions));
        } else {
            // No subscriptions - return empty feed
            return Ok(vec![]);
        }

        // Apply content filters
        if params.exclude_deleted {
            query = query.filter(posts::is_deleted.eq(false));
        }

        if params.exclude_removed {
            query = query.filter(posts::is_removed.eq(false));
        }

        if !params.nsfw {
            query = query.filter(posts::is_nsfw.eq(false));
        }

        // Apply sorting
        let query = match params.sort {
            SortType::Hot => query
                .order_by(post_aggregates::hot_rank.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::Active => query
                .order_by(post_aggregates::hot_rank_active.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::New => query.order_by(posts::creation_date.desc()),
            SortType::Old => query.order_by(posts::creation_date.asc()),
            SortType::TopDay => {
                let current_time = Utc::now().naive_utc();
                let day_ago = current_time - chrono::Duration::days(1);
                query
                    .filter(post_aggregates::creation_date.gt(day_ago))
                    .order_by(post_aggregates::score.desc())
                    .then_order_by(post_aggregates::creation_date.desc())
            }
            SortType::TopWeek => {
                let current_time = Utc::now().naive_utc();
                let week_ago = current_time - chrono::Duration::weeks(1);
                query
                    .filter(post_aggregates::creation_date.gt(week_ago))
                    .order_by(post_aggregates::score.desc())
                    .then_order_by(post_aggregates::creation_date.desc())
            }
            SortType::TopMonth => {
                let current_time = Utc::now().naive_utc();
                let month_ago = current_time - chrono::Duration::days(30);
                query
                    .filter(post_aggregates::creation_date.gt(month_ago))
                    .order_by(post_aggregates::score.desc())
                    .then_order_by(post_aggregates::creation_date.desc())
            }
            SortType::TopYear => {
                let current_time = Utc::now().naive_utc();
                let year_ago = current_time - chrono::Duration::days(365);
                query
                    .filter(post_aggregates::creation_date.gt(year_ago))
                    .order_by(post_aggregates::score.desc())
                    .then_order_by(post_aggregates::creation_date.desc())
            }
            SortType::TopAll => query
                .order_by(post_aggregates::score.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::MostComments => query
                .order_by(post_aggregates::comments.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::NewComments => query
                .order_by(post_aggregates::newest_comment_time.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::Controversial => query
                .order_by(post_aggregates::controversy_rank.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
        };

        // Apply pagination
        let (limit, offset) = limit_and_offset(params.page, params.limit)?;
        let query = query.limit(limit).offset(offset);

        // Execute query
        query
            .load::<(Post, PostAggregates)>(conn)
            .await
    }

    /// Get a count of posts that would be in the feed (for pagination)
    pub async fn get_feed_count(pool: &DbPool, stream_id: i32) -> Result<i64, Error> {
        let conn = &mut get_conn(pool).await?;

        // Get board IDs where stream wants all posts
        let full_board_ids: Vec<i32> = stream_board_subscriptions::table
            .filter(stream_board_subscriptions::stream_id.eq(stream_id))
            .filter(stream_board_subscriptions::include_all_posts.eq(true))
            .select(stream_board_subscriptions::board_id)
            .load::<i32>(conn)
            .await?;

        // Get flair subscriptions
        let flair_subs: Vec<(i32, i32)> = stream_flair_subscriptions::table
            .filter(stream_flair_subscriptions::stream_id.eq(stream_id))
            .select((
                stream_flair_subscriptions::board_id,
                stream_flair_subscriptions::flair_id,
            ))
            .load::<(i32, i32)>(conn)
            .await?;

        // Build count query
        let mut query = posts::table.into_boxed();

        if !full_board_ids.is_empty() {
            query = query.filter(posts::board_id.eq_any(&full_board_ids));
        } else if !flair_subs.is_empty() {
            // Extract just the board IDs for now (until flair_id is added to posts table)
            let flair_board_ids: Vec<i32> = flair_subs.iter().map(|(board_id, _)| *board_id).collect();
            query = query.filter(posts::board_id.eq_any(flair_board_ids));
        } else {
            return Ok(0);
        }

        query
            .filter(posts::is_deleted.eq(false))
            .filter(posts::is_removed.eq(false))
            .count()
            .get_result::<i64>(conn)
            .await
    }

    /// Check if a stream has any subscriptions
    pub async fn has_subscriptions(pool: &DbPool, stream_id: i32) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;

        let board_count = stream_board_subscriptions::table
            .filter(stream_board_subscriptions::stream_id.eq(stream_id))
            .count()
            .get_result::<i64>(conn)
            .await?;

        if board_count > 0 {
            return Ok(true);
        }

        let flair_count = stream_flair_subscriptions::table
            .filter(stream_flair_subscriptions::stream_id.eq(stream_id))
            .count()
            .get_result::<i64>(conn)
            .await?;

        Ok(flair_count > 0)
    }
}

/// Helper function to generate a stream feed with default parameters
pub async fn generate_stream_feed(
    pool: &DbPool,
    stream_id: i32,
    sort: SortType,
    page: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<(Post, PostAggregates)>, Error> {
    let params = StreamFeedParams {
        stream_id,
        sort,
        page,
        limit,
        ..Default::default()
    };

    StreamFeedGenerator::generate_feed(pool, &params).await
}
