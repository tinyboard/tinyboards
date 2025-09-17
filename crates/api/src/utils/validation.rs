use tinyboards_db::{
    models::{site::site::Site, user::user_blocks::UserBlock},
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

/// Check if downvotes are enabled for the site
#[tracing::instrument(skip_all)]
pub async fn check_downvotes_enabled(score: i32, pool: &DbPool) -> Result<(), TinyBoardsError> {
    if score == -1 {
        let site = Site::read(pool).await?;

        if !site.enable_downvotes {
            return Err(TinyBoardsError::from_message(403, "downvotes are disabled"));
        }
    }
    Ok(())
}

/// Check if one user has blocked another
#[tracing::instrument(skip_all)]
pub async fn check_person_block(
    my_id: i32,
    other_id: i32,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    let is_blocked = UserBlock::is_blocked(pool, other_id, my_id)
        .await
        .unwrap_or(false);

    if is_blocked {
        Err(TinyBoardsError::from_message(405, "user is blocking you"))
    } else {
        Ok(())
    }
}