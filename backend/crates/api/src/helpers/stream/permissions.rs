use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        stream::Stream,
        user::user::User,
    },
    schema::{stream_followers, streams},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;

/// Check if a user can view a stream
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
    let conn = &mut get_conn(pool).await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    let is_following: bool = stream_followers::table
        .filter(stream_followers::stream_id.eq(stream.id))
        .filter(stream_followers::user_id.eq(user.id))
        .count()
        .get_result::<i64>(conn)
        .await
        .map(|c| c > 0)
        .unwrap_or(false);

    if is_following {
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

    if user.has_permission(AdminPerms::Content) {
        return Ok(());
    }

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

    let conn = &mut get_conn(pool).await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    let count: i64 = streams::table
        .filter(streams::creator_id.eq(user.id))
        .count()
        .get_result(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

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
    use tinyboards_db::schema::{stream_flair_subscriptions, stream_board_subscriptions};

    const MAX_SUBSCRIPTIONS_PER_STREAM: i64 = 100;

    let conn = &mut get_conn(pool).await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    let flair_count: i64 = stream_flair_subscriptions::table
        .filter(stream_flair_subscriptions::stream_id.eq(stream.id))
        .count()
        .get_result(conn)
        .await
        .unwrap_or(0);

    let board_count: i64 = stream_board_subscriptions::table
        .filter(stream_board_subscriptions::stream_id.eq(stream.id))
        .count()
        .get_result(conn)
        .await
        .unwrap_or(0);

    let total = flair_count + board_count + additional_count as i64;

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
