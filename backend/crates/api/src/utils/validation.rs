use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    schema::{site, user_blocks},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

/// Check if downvotes are enabled for the site
#[tracing::instrument(skip_all)]
pub async fn check_downvotes_enabled(score: i32, pool: &DbPool) -> Result<(), TinyBoardsError> {
    if score == -1 {
        let conn = &mut get_conn(pool).await?;

        let enable_downvotes: bool = site::table
            .select(site::enable_downvotes)
            .first(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if !enable_downvotes {
            return Err(TinyBoardsError::from_message(403, "downvotes are disabled"));
        }
    }
    Ok(())
}

/// Check if one user has blocked another
#[tracing::instrument(skip_all)]
pub async fn check_person_block(
    my_id: Uuid,
    other_id: Uuid,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    let conn = &mut get_conn(pool).await?;

    let is_blocked: bool = user_blocks::table
        .filter(user_blocks::user_id.eq(other_id))
        .filter(user_blocks::target_id.eq(my_id))
        .count()
        .get_result::<i64>(conn)
        .await
        .map(|c| c > 0)
        .unwrap_or(false);

    if is_blocked {
        Err(TinyBoardsError::from_message(405, "user is blocking you"))
    } else {
        Ok(())
    }
}
