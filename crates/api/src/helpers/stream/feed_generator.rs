use tinyboards_db::{
    models::{
        stream::{
            stream::Stream,
            stream_flair_subscription::StreamFlairSubscription,
            stream_board_subscription::StreamBoardSubscription,
        },
        user::user::User,
        post::posts::Post as DbPost,
        flair::flair_template::FlairTemplate,
        board::boards::Board,
    },
    utils::DbPool,
    SortType as DbSortType,
    traits::Crud,
};
use tinyboards_utils::TinyBoardsError;
use crate::structs::post::Post;

/// Generate a feed of posts for a stream
///
/// This is the core feature that combines:
/// 1. Posts with flairs that are in the stream's flair subscriptions
/// 2. ALL posts from boards that are in the stream's board subscriptions
///
/// The feed respects:
/// - User NSFW preferences (if logged in)
/// - Sorting (Hot, New, Top with time ranges, Active)
/// - Pagination
/// - Deleted/removed post filtering
/// - max_posts_per_board limiting (if configured)
pub async fn generate_stream_feed(
    pool: &DbPool,
    stream: &Stream,
    user: Option<&User>,
    sort_type: DbSortType,
    limit: i64,
    offset: i64,
) -> Result<Vec<Post>, async_graphql::Error> {
    // Get flair subscriptions
    let flair_subs = StreamFlairSubscription::list_for_stream(pool, stream.id)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load flair subscriptions"))?;

    // Get board subscriptions
    let board_subs = StreamBoardSubscription::list_for_stream(pool, stream.id)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load board subscriptions"))?;

    // Extract flair IDs and board IDs
    let flair_ids: Vec<i32> = flair_subs.iter().map(|s| s.flair_id).collect();
    let board_ids: Vec<i32> = board_subs.iter().map(|s| s.board_id).collect();

    // If no subscriptions, return empty feed
    if flair_ids.is_empty() && board_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Determine NSFW filter
    let show_nsfw = user.map(|u| u.show_nsfw).unwrap_or(false);

    // User ID for permission checks (-1 for anonymous)
    let user_id = user.map(|u| u.id).unwrap_or(-1);

    // Query posts that match EITHER:
    // 1. Post has a flair in flair_ids
    // 2. Post's board is in board_ids
    let posts_with_aggregates = DbPost::load_stream_feed(
        pool,
        user_id,
        &flair_ids,
        &board_ids,
        sort_type,
        show_nsfw,
        stream.max_posts_per_board,
        limit,
        offset,
    )
    .await
    .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to generate stream feed"))?;

    // Convert to GraphQL Post types
    let posts: Vec<Post> = posts_with_aggregates
        .into_iter()
        .map(|(post, aggregates)| Post::from((post, aggregates)))
        .collect();

    Ok(posts)
}

/// Helper to validate flair and board subscriptions before adding
pub async fn validate_subscriptions(
    pool: &DbPool,
    flair_ids: &[i32],
    board_ids: &[i32],
) -> Result<(), async_graphql::Error> {
    // Validate flair IDs exist (flair_ids refer to FlairTemplate IDs)
    for &flair_id in flair_ids {
        <FlairTemplate as Crud>::read(pool, flair_id)
            .await
            .map_err(|_| TinyBoardsError::from_message(404, &format!("Flair template with ID {} not found", flair_id)))?;
    }

    // Validate board IDs exist and are not deleted/banned
    for &board_id in board_ids {
        let board = <Board as Crud>::read(pool, board_id)
            .await
            .map_err(|_| TinyBoardsError::from_message(404, &format!("Board with ID {} not found", board_id)))?;

        if board.is_deleted || board.is_removed {
            return Err(TinyBoardsError::from_message(
                410,
                &format!("Board with ID {} is not available", board_id)
            ).into());
        }
    }

    Ok(())
}

/// Check for duplicate subscriptions
pub async fn check_duplicate_flair_subscription(
    pool: &DbPool,
    stream_id: i32,
    flair_id: i32,
) -> Result<(), async_graphql::Error> {
    if StreamFlairSubscription::exists(pool, stream_id, flair_id).await.unwrap_or(false) {
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
    stream_id: i32,
    board_id: i32,
) -> Result<(), async_graphql::Error> {
    if StreamBoardSubscription::exists(pool, stream_id, board_id).await.unwrap_or(false) {
        return Err(TinyBoardsError::from_message(
            409,
            "This board is already subscribed in this stream"
        ).into());
    }
    Ok(())
}

/// Get boards associated with flair subscriptions
pub async fn get_boards_for_flair_subscriptions(
    pool: &DbPool,
    flair_ids: &[i32],
) -> Result<Vec<i32>, async_graphql::Error> {
    let mut board_ids = Vec::new();

    for &flair_id in flair_ids {
        if let Ok(flair) = <FlairTemplate as Crud>::read(pool, flair_id).await {
            board_ids.push(flair.board_id);
        }
    }

    Ok(board_ids)
}
