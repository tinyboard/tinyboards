use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_auth::tokens::validate_access_token;
use tinyboards_db::{
    models::auth::Secret,
    schema::users,
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;

/// Extract user from Authorization header (Bearer token).
///
/// Uses the auth crate's token validation so that the Claims format
/// matches what the login/register endpoints produce.
pub async fn get_user_from_header_opt(
    pool: &DbPool,
    master_key: &Secret,
    auth: Option<&str>,
) -> Result<Option<tinyboards_db::models::user::User>, TinyBoardsError> {
    let auth = match auth {
        Some(a) if !a.is_empty() => a,
        _ => return Ok(None),
    };

    if !auth.starts_with("Bearer ") {
        return Err(TinyBoardsError::from_message(
            400,
            "Invalid `Authorization` header! It should be `Authorization: Bearer <access token>`",
        ));
    }

    let token = &auth[7..];

    let claims = validate_access_token(token, &master_key.jwt_secret)
        .map_err(|_| TinyBoardsError::from_message(401, "Invalid or expired token"))?;

    let user_uuid = claims.sub;

    // Load user from database
    let conn = &mut get_conn(pool).await?;
    let user: tinyboards_db::models::user::User = users::table
        .find(user_uuid)
        .first(conn)
        .await
        .map_err(|_| TinyBoardsError::from_message(401, "User not found"))?;

    // Update last_seen timestamp (fire and forget)
    let pool_clone = pool.clone();
    let user_id = user.id;
    tokio::spawn(async move {
        if let Ok(conn) = &mut get_conn(&pool_clone).await {
            let _ = diesel::update(users::table.find(user_id))
                .set(users::updated_at.eq(Utc::now()))
                .execute(conn)
                .await;
        }
    });

    Ok(Some(user))
}

/// Checks the password length
pub fn password_length_check(pass: &str) -> Result<(), TinyBoardsError> {
    if !(10..=60).contains(&pass.len()) {
        Err(TinyBoardsError::from_message(
            400,
            "password length must be between 10-60 characters",
        ))
    } else {
        Ok(())
    }
}
