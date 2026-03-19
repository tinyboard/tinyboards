use diesel::sql_query;
use diesel::sql_types::Text;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::errors::AuthError;
use crate::types::{AuthSessionRow, AuthUser, CreatedUser, EmailVerificationRow, JwtSecretRow, PasswordResetRow, SiteRegistrationInfo};

/// Type alias for the async connection pool.
pub type DbPool = diesel_async::pooled_connection::bb8::Pool<diesel_async::AsyncPgConnection>;

/// Get a connection from the pool.
async fn get_conn(
    pool: &DbPool,
) -> Result<
    bb8::PooledConnection<'_, diesel_async::pooled_connection::AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>>,
    AuthError,
> {
    pool.get().await.map_err(|e| AuthError::DatabaseError(format!("Pool error: {}", e)))
}

// ============================================================
// JWT Secret
// ============================================================

/// Load the JWT secret from the secrets table.
pub async fn get_jwt_secret(pool: &DbPool) -> Result<String, AuthError> {
    let conn = &mut get_conn(pool).await?;
    let row: JwtSecretRow = sql_query("SELECT jwt_secret FROM secrets LIMIT 1")
        .get_result(conn)
        .await
        .map_err(|e| AuthError::DatabaseError(format!("Failed to load JWT secret: {}", e)))?;
    Ok(row.jwt_secret)
}

// ============================================================
// User lookups
// ============================================================

const AUTH_USER_COLUMNS: &str =
    "id, name, email, passhash, is_email_verified, is_banned, is_admin, admin_level, is_application_accepted, deleted_at";

/// Look up a user by username (case-insensitive).
pub async fn get_user_by_name(pool: &DbPool, username: &str) -> Result<AuthUser, AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query(format!(
        "SELECT {} FROM users WHERE lower(name) = lower($1) LIMIT 1",
        AUTH_USER_COLUMNS
    ))
    .bind::<Text, _>(username)
    .get_result(conn)
    .await
    .map_err(|_| AuthError::InvalidCredentials)
}

/// Look up a user by email.
pub async fn get_user_by_email(pool: &DbPool, email: &str) -> Result<AuthUser, AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query(format!(
        "SELECT {} FROM users WHERE email = $1 LIMIT 1",
        AUTH_USER_COLUMNS
    ))
    .bind::<Text, _>(email)
    .get_result(conn)
    .await
    .map_err(|_| AuthError::InvalidCredentials)
}

/// Look up a user by ID.
pub async fn get_user_by_id(pool: &DbPool, user_id: Uuid) -> Result<AuthUser, AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query(format!(
        "SELECT {} FROM users WHERE id = $1 LIMIT 1",
        AUTH_USER_COLUMNS
    ))
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .get_result(conn)
    .await
    .map_err(|_| AuthError::InvalidCredentials)
}

// ============================================================
// User creation (registration)
// ============================================================

/// Create a new user and return their id and name.
pub async fn create_user(
    pool: &DbPool,
    username: &str,
    display_name: Option<&str>,
    email: Option<&str>,
    passhash: &str,
    is_application_accepted: bool,
) -> Result<CreatedUser, AuthError> {
    let conn = &mut get_conn(pool).await?;
    let display = display_name.unwrap_or(username);

    sql_query(
        "INSERT INTO users (name, display_name, email, passhash, is_application_accepted)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, name"
    )
    .bind::<Text, _>(username)
    .bind::<Text, _>(display)
    .bind::<diesel::sql_types::Nullable<Text>, _>(email)
    .bind::<Text, _>(passhash)
    .bind::<diesel::sql_types::Bool, _>(is_application_accepted)
    .get_result(conn)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("users_email_key") || msg.contains("unique") && msg.contains("email") {
            AuthError::DuplicateUser("email address".to_string())
        } else if msg.contains("users_name") || msg.contains("unique") && msg.contains("name") {
            AuthError::DuplicateUser("username".to_string())
        } else {
            AuthError::DatabaseError(format!("Failed to create user: {}", e))
        }
    })
}

/// Update user's passhash.
pub async fn update_user_passhash(
    pool: &DbPool,
    user_id: Uuid,
    new_passhash: &str,
) -> Result<(), AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query("UPDATE users SET passhash = $1, updated_at = now() WHERE id = $2")
        .bind::<Text, _>(new_passhash)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .execute(conn)
        .await
        .map_err(|e| AuthError::DatabaseError(format!("Failed to update password: {}", e)))?;
    Ok(())
}

/// Mark user's email as verified.
pub async fn set_email_verified(pool: &DbPool, user_id: Uuid) -> Result<(), AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query("UPDATE users SET is_email_verified = true, updated_at = now() WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .execute(conn)
        .await
        .map_err(|e| AuthError::DatabaseError(format!("Failed to verify email: {}", e)))?;
    Ok(())
}

// ============================================================
// Auth sessions (refresh token management)
// ============================================================

/// Create a new auth session and return the session ID.
pub async fn create_session(
    pool: &DbPool,
    user_id: Uuid,
    refresh_token_hash: &str,
    user_agent: Option<&str>,
    ip_address: Option<&str>,
) -> Result<Uuid, AuthError> {
    let conn = &mut get_conn(pool).await?;

    #[derive(diesel::QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    let row: IdRow = sql_query(
        "INSERT INTO auth_sessions (user_id, refresh_token_hash, user_agent, ip_address, expires_at)
         VALUES ($1, $2, $3, $4, NOW() + INTERVAL '30 days')
         RETURNING id"
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<Text, _>(refresh_token_hash)
    .bind::<diesel::sql_types::Nullable<Text>, _>(user_agent)
    .bind::<diesel::sql_types::Nullable<Text>, _>(ip_address)
    .get_result(conn)
    .await
    .map_err(|e| AuthError::DatabaseError(format!("Failed to create session: {}", e)))?;

    Ok(row.id)
}

/// Get all active (non-expired) sessions for a user.
pub async fn get_active_sessions(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Vec<AuthSessionRow>, AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query(
        "SELECT id, user_id, refresh_token_hash, user_agent, ip_address, last_used_at, expires_at, created_at
         FROM auth_sessions
         WHERE user_id = $1 AND expires_at > NOW()"
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .get_results(conn)
    .await
    .map_err(|e| AuthError::DatabaseError(format!("Failed to get sessions: {}", e)))
}

/// Rotate a session's refresh token (update hash, extend expiry).
pub async fn rotate_session(
    pool: &DbPool,
    session_id: Uuid,
    new_refresh_token_hash: &str,
) -> Result<(), AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query(
        "UPDATE auth_sessions
         SET refresh_token_hash = $1, last_used_at = NOW(), expires_at = NOW() + INTERVAL '30 days'
         WHERE id = $2"
    )
    .bind::<Text, _>(new_refresh_token_hash)
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .execute(conn)
    .await
    .map_err(|e| AuthError::DatabaseError(format!("Failed to rotate session: {}", e)))?;
    Ok(())
}

/// Delete a single session (logout).
pub async fn delete_session(
    pool: &DbPool,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<(), AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query("DELETE FROM auth_sessions WHERE id = $1 AND user_id = $2")
        .bind::<diesel::sql_types::Uuid, _>(session_id)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .execute(conn)
        .await
        .map_err(|e| AuthError::DatabaseError(format!("Failed to delete session: {}", e)))?;
    Ok(())
}

/// Delete ALL sessions for a user (logout all devices).
pub async fn delete_all_sessions(pool: &DbPool, user_id: Uuid) -> Result<usize, AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query("DELETE FROM auth_sessions WHERE user_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .execute(conn)
        .await
        .map_err(|e| AuthError::DatabaseError(format!("Failed to delete sessions: {}", e)))
}

/// Clean up expired sessions (background task).
pub async fn cleanup_expired_sessions(pool: &DbPool) -> Result<usize, AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query("DELETE FROM auth_sessions WHERE expires_at < NOW()")
        .execute(conn)
        .await
        .map_err(|e| AuthError::DatabaseError(format!("Failed to cleanup sessions: {}", e)))
}

// ============================================================
// Site configuration
// ============================================================

/// Get the site's registration mode.
pub async fn get_registration_mode(pool: &DbPool) -> Result<SiteRegistrationInfo, AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query(
        "SELECT registration_mode::text AS registration_mode, application_question
         FROM site LIMIT 1"
    )
    .get_result(conn)
    .await
    .map_err(|e| AuthError::DatabaseError(format!("Failed to get site config: {}", e)))
}

// ============================================================
// Registration support
// ============================================================

/// Validate and consume a site invite code.
pub async fn consume_invite(pool: &DbPool, invite_code: &str) -> Result<(), AuthError> {
    let conn = &mut get_conn(pool).await?;
    let affected = sql_query("DELETE FROM site_invites WHERE verification_code = $1")
        .bind::<Text, _>(invite_code)
        .execute(conn)
        .await
        .map_err(|e| AuthError::DatabaseError(format!("Failed to check invite: {}", e)))?;

    if affected == 0 {
        return Err(AuthError::InvalidInvite);
    }
    Ok(())
}

/// Create a registration application.
pub async fn create_application(
    pool: &DbPool,
    user_id: Uuid,
    answer: &str,
) -> Result<(), AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query(
        "INSERT INTO registration_applications (user_id, answer) VALUES ($1, $2)"
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<Text, _>(answer)
    .execute(conn)
    .await
    .map_err(|e| AuthError::DatabaseError(format!("Failed to create application: {}", e)))?;
    Ok(())
}

/// Check if user has a pending application.
pub async fn has_pending_application(pool: &DbPool, user_id: Uuid) -> Result<bool, AuthError> {
    let conn = &mut get_conn(pool).await?;

    #[derive(diesel::QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let row: CountRow = sql_query(
        "SELECT COUNT(*) as count FROM registration_applications
         WHERE user_id = $1 AND admin_id IS NULL AND deny_reason IS NULL"
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .get_result(conn)
    .await
    .map_err(|e| AuthError::DatabaseError(format!("Failed to check application: {}", e)))?;

    Ok(row.count > 0)
}

// ============================================================
// Password reset
// ============================================================

/// Create a password reset token (store hash in DB).
pub async fn create_password_reset(
    pool: &DbPool,
    user_id: Uuid,
    token_hash: &str,
) -> Result<(), AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query(
        "INSERT INTO password_resets (user_id, reset_token, expires_at)
         VALUES ($1, $2, NOW() + INTERVAL '1 hour')"
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<Text, _>(token_hash)
    .execute(conn)
    .await
    .map_err(|e| AuthError::DatabaseError(format!("Failed to create reset token: {}", e)))?;
    Ok(())
}

/// Find valid (non-expired, non-used) password reset tokens for a user.
pub async fn get_valid_password_resets(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Vec<PasswordResetRow>, AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query(
        "SELECT id, user_id, reset_token, created_at, expires_at, used_at
         FROM password_resets
         WHERE user_id = $1 AND used_at IS NULL AND expires_at > NOW()"
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .get_results(conn)
    .await
    .map_err(|e| AuthError::DatabaseError(format!("Failed to get reset tokens: {}", e)))
}

/// Mark a password reset token as used.
pub async fn mark_reset_used(pool: &DbPool, reset_id: Uuid) -> Result<(), AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query("UPDATE password_resets SET used_at = NOW() WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(reset_id)
        .execute(conn)
        .await
        .map_err(|e| AuthError::DatabaseError(format!("Failed to mark reset as used: {}", e)))?;
    Ok(())
}

// ============================================================
// Email verification
// ============================================================

/// Create an email verification token (store hash in DB).
pub async fn create_email_verification(
    pool: &DbPool,
    user_id: Uuid,
    email: &str,
    token_hash: &str,
) -> Result<(), AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query(
        "INSERT INTO email_verification (user_id, email, verification_code)
         VALUES ($1, $2, $3)"
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<Text, _>(email)
    .bind::<Text, _>(token_hash)
    .execute(conn)
    .await
    .map_err(|e| AuthError::DatabaseError(format!("Failed to create verification: {}", e)))?;
    Ok(())
}

/// Find valid (unverified) email verification tokens for a user.
pub async fn get_pending_email_verifications(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Vec<EmailVerificationRow>, AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query(
        "SELECT id, user_id, email, verification_code, created_at, verified_at
         FROM email_verification
         WHERE user_id = $1 AND verified_at IS NULL
         ORDER BY created_at DESC"
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .get_results(conn)
    .await
    .map_err(|e| AuthError::DatabaseError(format!("Failed to get verifications: {}", e)))
}

/// Mark an email verification as complete.
pub async fn mark_email_verified(pool: &DbPool, verification_id: Uuid) -> Result<(), AuthError> {
    let conn = &mut get_conn(pool).await?;
    sql_query("UPDATE email_verification SET verified_at = NOW() WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(verification_id)
        .execute(conn)
        .await
        .map_err(|e| AuthError::DatabaseError(format!("Failed to mark verified: {}", e)))?;
    Ok(())
}

/// Find a user by email (for password reset lookups).
pub async fn find_user_by_email(
    pool: &DbPool,
    email: &str,
) -> Result<Option<AuthUser>, AuthError> {
    let conn = &mut get_conn(pool).await?;
    let result: Result<AuthUser, _> = sql_query(format!(
        "SELECT {} FROM users WHERE email = $1 LIMIT 1",
        AUTH_USER_COLUMNS
    ))
    .bind::<Text, _>(email)
    .get_result(conn)
    .await;

    match result {
        Ok(user) => Ok(Some(user)),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(e) => Err(AuthError::DatabaseError(format!("Failed to find user: {}", e))),
    }
}
