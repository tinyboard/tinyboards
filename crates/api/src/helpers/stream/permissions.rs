use tinyboards_db::{
    models::{
        stream::stream::Stream,
        stream::stream_follower::StreamFollower,
        user::user::User,
    },
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

/// Check if a user can view a stream
///
/// A user can view a stream if:
/// - The stream is public, OR
/// - The user is the creator, OR
/// - The user is following the stream, OR
/// - A valid share token is provided
pub async fn can_view_stream(
    user: Option<&User>,
    stream: &Stream,
    share_token: Option<&str>,
    pool: &DbPool,
) -> Result<(), async_graphql::Error> {
    // Public streams are viewable by anyone
    if stream.is_public {
        return Ok(());
    }

    // Check if valid share token provided
    if let Some(token) = share_token {
        if let Some(ref stream_token) = stream.share_token {
            if constant_time_compare(token, stream_token) {
                return Ok(());
            }
        }
    }

    // Require authentication for private streams
    let user = user.ok_or_else(|| {
        TinyBoardsError::from_message(401, "Login required to view this private stream")
    })?;

    // Creator can always view their own streams
    if stream.creator_id == user.id {
        return Ok(());
    }

    // Check if user is following the stream
    if StreamFollower::is_following(pool, user.id, stream.id).await.unwrap_or(false) {
        return Ok(());
    }

    Err(TinyBoardsError::from_message(403, "You do not have permission to view this stream").into())
}

/// Check if user can edit a stream (creator only)
pub fn can_edit_stream(
    user: Option<&User>,
    stream: &Stream,
) -> Result<(), async_graphql::Error> {
    let user = user.ok_or_else(|| {
        TinyBoardsError::from_message(401, "Login required")
    })?;

    if stream.creator_id != user.id {
        return Err(TinyBoardsError::from_message(403, "Only the stream creator can edit this stream").into());
    }

    Ok(())
}

/// Check if user can delete a stream (creator or admin)
pub fn can_delete_stream(
    user: Option<&User>,
    stream: &Stream,
) -> Result<(), async_graphql::Error> {
    use tinyboards_db::models::user::user::AdminPerms;

    let user = user.ok_or_else(|| {
        TinyBoardsError::from_message(401, "Login required")
    })?;

    // Admins can delete any stream
    if user.has_permission(AdminPerms::Content) {
        return Ok(());
    }

    // Creators can delete their own streams
    if stream.creator_id == user.id {
        return Ok(());
    }

    Err(TinyBoardsError::from_message(403, "You do not have permission to delete this stream").into())
}

/// Check stream creation quota (max 25 streams per user)
pub async fn check_stream_quota(
    user: &User,
    pool: &DbPool,
) -> Result<(), async_graphql::Error> {
    const MAX_STREAMS_PER_USER: i64 = 25;

    let count = Stream::get_user_stream_count(pool, user.id)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to check stream quota"))?;

    if count >= MAX_STREAMS_PER_USER {
        return Err(TinyBoardsError::from_message(
            429,
            &format!("You have reached the maximum limit of {} streams", MAX_STREAMS_PER_USER)
        ).into());
    }

    Ok(())
}

/// Check subscription quota (max 100 total subscriptions per stream)
pub async fn check_subscription_quota(
    stream: &Stream,
    pool: &DbPool,
    additional_count: usize,
) -> Result<(), async_graphql::Error> {
    use tinyboards_db::aggregates::structs::StreamAggregates;

    const MAX_SUBSCRIPTIONS_PER_STREAM: i32 = 100;

    let aggregates = StreamAggregates::read(pool, stream.id)
        .await
        .unwrap_or_default();

    let total = aggregates.total_subscription_count + additional_count as i32;

    if total > MAX_SUBSCRIPTIONS_PER_STREAM {
        return Err(TinyBoardsError::from_message(
            429,
            &format!(
                "This stream has reached the maximum limit of {} subscriptions (flair + board subscriptions combined)",
                MAX_SUBSCRIPTIONS_PER_STREAM
            )
        ).into());
    }

    Ok(())
}

/// Constant-time string comparison to prevent timing attacks on share tokens
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
        result |= byte_a ^ byte_b;
    }

    result == 0
}
