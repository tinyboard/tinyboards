use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbSortType,
    models::{
        aggregates::PostAggregates,
        post::posts::Post as DbPost,
        stream::Stream,
        user::user::User,
    },
    schema::{
        boards, post_aggregates, posts,
        stream_board_subscriptions, stream_flair_subscriptions,
    },
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::structs::post::Post;

/// Generate a feed of posts for a stream
///
/// Combines posts from board subscriptions and flair subscriptions.
/// Note: flair-based filtering uses stream_flair_subscriptions.board_id
/// to match boards, since flair_id types are pending schema migration.
pub async fn generate_stream_feed(
    pool: &DbPool,
    stream: &Stream,
    user: Option<&User>,
    _sort_type: DbSortType,
    _post_type_filter: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Post>, async_graphql::Error> {
    let conn = &mut get_conn(pool).await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    // Get board IDs from board subscriptions
    let board_sub_ids: Vec<Uuid> = stream_board_subscriptions::table
        .filter(stream_board_subscriptions::stream_id.eq(stream.id))
        .select(stream_board_subscriptions::board_id)
        .load(conn)
        .await
        .unwrap_or_default();

    // Get board IDs from flair subscriptions (each flair sub is scoped to a board)
    let flair_sub_board_ids: Vec<Uuid> = stream_flair_subscriptions::table
        .filter(stream_flair_subscriptions::stream_id.eq(stream.id))
        .select(stream_flair_subscriptions::board_id)
        .load(conn)
        .await
        .unwrap_or_default();

    // Combine all board IDs (deduplicated)
    let mut all_board_ids: Vec<Uuid> = board_sub_ids;
    for bid in flair_sub_board_ids {
        if !all_board_ids.contains(&bid) {
            all_board_ids.push(bid);
        }
    }

    if all_board_ids.is_empty() {
        return Ok(Vec::new());
    }

    let show_nsfw = user.map(|u| u.show_nsfw).unwrap_or(false);

    // Build the main post query
    let mut query = posts::table
        .inner_join(post_aggregates::table.on(post_aggregates::post_id.eq(posts::id)))
        .filter(posts::deleted_at.is_null())
        .filter(posts::is_removed.eq(false))
        .filter(posts::board_id.eq_any(&all_board_ids))
        .into_boxed();

    if !show_nsfw {
        query = query.filter(posts::is_nsfw.eq(false));
    }

    // Apply sorting
    query = query.order(posts::created_at.desc());

    let results: Vec<(DbPost, PostAggregates)> = query
        .limit(limit)
        .offset(offset)
        .load(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    let posts_out: Vec<Post> = results
        .into_iter()
        .map(|(post, aggregates)| Post::from((post, aggregates)))
        .collect();

    Ok(posts_out)
}

/// Validate board subscriptions before adding
pub async fn validate_board_subscriptions(
    pool: &DbPool,
    board_ids: &[Uuid],
) -> Result<(), async_graphql::Error> {
    let conn = &mut get_conn(pool).await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    for &board_id in board_ids {
        let board_exists: bool = boards::table
            .find(board_id)
            .filter(boards::deleted_at.is_null())
            .filter(boards::is_removed.eq(false))
            .count()
            .get_result::<i64>(conn)
            .await
            .map(|c| c > 0)
            .unwrap_or(false);

        if !board_exists {
            return Err(TinyBoardsError::from_message(
                404,
                &format!("Board with ID {} not found or unavailable", board_id)
            ).into());
        }
    }

    Ok(())
}

/// Check for duplicate flair subscription
pub async fn check_duplicate_flair_subscription(
    pool: &DbPool,
    stream_id: Uuid,
    flair_id: i32,
) -> Result<(), async_graphql::Error> {
    let conn = &mut get_conn(pool).await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    let exists: bool = stream_flair_subscriptions::table
        .filter(stream_flair_subscriptions::stream_id.eq(stream_id))
        .filter(stream_flair_subscriptions::flair_id.eq(flair_id))
        .count()
        .get_result::<i64>(conn)
        .await
        .map(|c| c > 0)
        .unwrap_or(false);

    if exists {
        return Err(TinyBoardsError::from_message(
            409,
            "This flair is already subscribed in this stream"
        ).into());
    }
    Ok(())
}

/// Check for duplicate board subscription
pub async fn check_duplicate_board_subscription(
    pool: &DbPool,
    stream_id: Uuid,
    board_id: Uuid,
) -> Result<(), async_graphql::Error> {
    let conn = &mut get_conn(pool).await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    let exists: bool = stream_board_subscriptions::table
        .filter(stream_board_subscriptions::stream_id.eq(stream_id))
        .filter(stream_board_subscriptions::board_id.eq(board_id))
        .count()
        .get_result::<i64>(conn)
        .await
        .map(|c| c > 0)
        .unwrap_or(false);

    if exists {
        return Err(TinyBoardsError::from_message(
            409,
            "This board is already subscribed in this stream"
        ).into());
    }
    Ok(())
}
