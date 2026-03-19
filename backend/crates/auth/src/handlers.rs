use actix_web::{web, HttpRequest, HttpResponse};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures::future::{ok, LocalBoxFuture, Ready};
use regex::Regex;
use std::rc::Rc;
use std::task::{Context, Poll};
use uuid::Uuid;

use crate::cookies;
use crate::errors::AuthError;
use crate::middleware::AuthExt;
use crate::password;
use crate::session::{self, DbPool};
use crate::tokens;
use crate::types::*;

// ============================================================
// CSRF Guard — validates X-Requested-With: XMLHttpRequest
// ============================================================

/// Middleware that rejects POST requests without a valid X-Requested-With header.
/// This is a simple stateless CSRF mitigation for the auth endpoints.
pub struct CsrfGuard;

impl<S, B> Transform<S, ServiceRequest> for CsrfGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CsrfGuardService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CsrfGuardService {
            service: Rc::new(service),
        })
    }
}

pub struct CsrfGuardService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for CsrfGuardService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // Only enforce on POST requests (all auth endpoints are POST)
            if req.method() == actix_web::http::Method::POST {
                let has_header = req
                    .headers()
                    .get("x-requested-with")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.eq_ignore_ascii_case("xmlhttprequest"))
                    .unwrap_or(false);

                if !has_header {
                    return Err(AuthError::CsrfViolation.into());
                }
            }

            service.call(req).await
        })
    }
}

// ============================================================
// Login
// ============================================================

pub async fn login(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AuthError> {
    // Look up user by email or username
    let user = if body.username_or_email.contains('@') {
        session::get_user_by_email(&pool, &body.username_or_email).await?
    } else {
        session::get_user_by_name(&pool, &body.username_or_email).await?
    };

    // Check if account is deleted
    if user.deleted_at.is_some() {
        return Err(AuthError::InvalidCredentials);
    }

    // Verify password
    let password_valid = password::verify_password(&body.password, &user.passhash)?;
    if !password_valid {
        return Err(AuthError::InvalidCredentials);
    }

    // Check if banned
    if user.is_banned {
        return Err(AuthError::AccountBanned);
    }

    // Check for pending application
    if !user.is_application_accepted {
        let has_pending = session::has_pending_application(&pool, user.id).await?;
        if has_pending {
            return Err(AuthError::ApplicationPending);
        }
    }

    // Create session and tokens
    let jwt_secret = session::get_jwt_secret(&pool).await?;
    let role = UserRole::from_admin_fields(user.is_admin, user.admin_level);

    let access_token = tokens::create_access_token(user.id, role, &jwt_secret)?;
    let refresh_token = tokens::generate_refresh_token();
    let refresh_hash = tokens::hash_refresh_token(&refresh_token);

    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let ip_address = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    session::create_session(
        &pool,
        user.id,
        &refresh_hash,
        user_agent.as_deref(),
        ip_address.as_deref(),
    )
    .await?;

    let mut response = HttpResponse::Ok().json(AuthResponse {
        success: true,
        message: None,
        user: Some(UserInfo {
            id: user.id,
            name: user.name,
            is_admin: user.is_admin,
            admin_level: user.admin_level,
        }),
    });

    cookies::set_auth_cookies(&mut response, &access_token, &refresh_token);

    Ok(response)
}

// ============================================================
// Register
// ============================================================

pub async fn register(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AuthError> {
    // Get site registration mode
    let site_info = session::get_registration_mode(&pool).await?;
    let reg_mode = site_info.registration_mode.as_str();

    // Check registration policy constraints
    match reg_mode {
        "closed" => return Err(AuthError::RegistrationClosed),
        "invite_only" => {
            if body.invite_code.is_none() {
                return Err(AuthError::InviteRequired);
            }
        }
        "application_required" => {
            if body.application_answer.is_none() {
                return Err(AuthError::ApplicationRequired);
            }
        }
        // "open" and other modes pass through
        _ => {}
    }

    // Validate username format
    let re = Regex::new(r"^[A-Za-z][A-Za-z0-9_]{0,29}$")
        .map_err(|e| AuthError::InternalError(format!("Regex error: {}", e)))?;
    if !re.is_match(&body.username) {
        return Err(AuthError::InvalidUsername);
    }

    // Validate password length
    password::validate_password_length(&body.password)?;

    // If invite mode, validate and consume invite
    if reg_mode == "invite_only" {
        if let Some(ref code) = body.invite_code {
            session::consume_invite(&pool, code).await?;
        }
    }

    // Hash password
    let passhash = password::hash_password(&body.password)?;

    // Determine if the account is immediately accepted
    let requires_application = reg_mode == "application_required";
    let is_accepted = !requires_application;

    // Create user
    let created_user = session::create_user(
        &pool,
        &body.username,
        body.display_name.as_deref(),
        body.email.as_deref(),
        &passhash,
        is_accepted,
    )
    .await?;

    // If application mode, create the application
    if requires_application {
        if let Some(ref answer) = body.application_answer {
            session::create_application(&pool, created_user.id, answer).await?;
        }
    }

    // For application mode, don't log them in
    if requires_application {
        return Ok(HttpResponse::Ok().json(RegisterResponse {
            success: true,
            account_created: false,
            application_submitted: true,
            user: None,
            message: Some("Your application has been submitted. You will be notified when it is reviewed.".to_string()),
        }));
    }

    // For all other modes, create session and log them in
    let jwt_secret = session::get_jwt_secret(&pool).await?;
    let role = UserRole::User; // New accounts are never admin

    let access_token = tokens::create_access_token(created_user.id, role, &jwt_secret)?;
    let refresh_token = tokens::generate_refresh_token();
    let refresh_hash = tokens::hash_refresh_token(&refresh_token);

    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let ip_address = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    session::create_session(
        &pool,
        created_user.id,
        &refresh_hash,
        user_agent.as_deref(),
        ip_address.as_deref(),
    )
    .await?;

    let mut response = HttpResponse::Ok().json(RegisterResponse {
        success: true,
        account_created: true,
        application_submitted: false,
        user: Some(UserInfo {
            id: created_user.id,
            name: created_user.name,
            is_admin: false,
            admin_level: 0,
        }),
        message: None,
    });

    cookies::set_auth_cookies(&mut response, &access_token, &refresh_token);

    Ok(response)
}

// ============================================================
// Logout
// ============================================================

pub async fn logout(
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> Result<HttpResponse, AuthError> {
    let auth_user = req.require_auth()?;

    // Find the session to delete by checking the refresh token
    if let Some(cookie) = req.cookie(cookies::REFRESH_COOKIE_NAME) {
        let raw_token = cookie.value();
        let sessions = session::get_active_sessions(&pool, auth_user.id).await?;

        for sess in &sessions {
            if tokens::verify_refresh_token(raw_token, &sess.refresh_token_hash) {
                session::delete_session(&pool, sess.id, auth_user.id).await?;
                break;
            }
        }
    }

    let mut response = HttpResponse::Ok().json(AuthResponse {
        success: true,
        message: Some("Logged out successfully".to_string()),
        user: None,
    });

    cookies::clear_auth_cookies(&mut response);
    Ok(response)
}

// ============================================================
// Logout all devices
// ============================================================

pub async fn logout_all(
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> Result<HttpResponse, AuthError> {
    let auth_user = req.require_auth()?;

    let count = session::delete_all_sessions(&pool, auth_user.id).await?;

    let mut response = HttpResponse::Ok().json(AuthResponse {
        success: true,
        message: Some(format!("Logged out from {} device(s)", count)),
        user: None,
    });

    cookies::clear_auth_cookies(&mut response);
    Ok(response)
}

// ============================================================
// Refresh token
// ============================================================

pub async fn refresh_token(
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> Result<HttpResponse, AuthError> {
    // Get the refresh token from the cookie
    let refresh_cookie = req
        .cookie(cookies::REFRESH_COOKIE_NAME)
        .ok_or(AuthError::InvalidRefreshToken)?;

    let raw_token = refresh_cookie.value();

    // We need to find which session this token belongs to.
    // Since we don't know the user yet, we try to extract it from the (possibly expired) access token.
    // If no access token, the refresh still works if we can match the token.

    // First try to get user from access token (even if expired, we just need the sub)
    let jwt_secret = session::get_jwt_secret(&pool).await?;

    let user_id: Uuid = if let Some(access_cookie) = req.cookie(cookies::ACCESS_COOKIE_NAME) {
        // Try to decode without expiry validation to get the user ID
        let key = jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes());
        let mut validation = jsonwebtoken::Validation::default();
        validation.validate_exp = false; // Allow expired tokens for refresh
        validation.required_spec_claims.clear();

        match jsonwebtoken::decode::<crate::claims::Claims>(access_cookie.value(), &key, &validation) {
            Ok(token_data) => token_data.claims.sub,
            Err(_) => return Err(AuthError::InvalidRefreshToken),
        }
    } else {
        return Err(AuthError::InvalidRefreshToken);
    };

    // Find matching session
    let sessions = session::get_active_sessions(&pool, user_id).await?;
    let matching_session = sessions
        .iter()
        .find(|s| tokens::verify_refresh_token(raw_token, &s.refresh_token_hash))
        .ok_or(AuthError::InvalidRefreshToken)?;

    let session_id = matching_session.id;

    // Generate new token pair
    let user = session::get_user_by_id(&pool, user_id).await?;
    let role = UserRole::from_admin_fields(user.is_admin, user.admin_level);

    let new_access_token = tokens::create_access_token(user.id, role, &jwt_secret)?;
    let new_refresh_token = tokens::generate_refresh_token();
    let new_refresh_hash = tokens::hash_refresh_token(&new_refresh_token);

    // Rotate the session
    session::rotate_session(&pool, session_id, &new_refresh_hash).await?;

    let mut response = HttpResponse::Ok().json(AuthResponse {
        success: true,
        message: None,
        user: Some(UserInfo {
            id: user.id,
            name: user.name,
            is_admin: user.is_admin,
            admin_level: user.admin_level,
        }),
    });

    cookies::set_auth_cookies(&mut response, &new_access_token, &new_refresh_token);

    Ok(response)
}

// ============================================================
// Change password
// ============================================================

pub async fn change_password(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    body: web::Json<ChangePasswordRequest>,
) -> Result<HttpResponse, AuthError> {
    let auth_user = req.require_auth()?;

    // Validate new password length
    password::validate_password_length(&body.new_password)?;

    // Get current user data to verify old password
    let user = session::get_user_by_id(&pool, auth_user.id).await?;

    // Verify current password
    let current_valid = password::verify_password(&body.current_password, &user.passhash)?;
    if !current_valid {
        return Err(AuthError::IncorrectPassword);
    }

    // Hash new password
    let new_hash = password::hash_password(&body.new_password)?;

    // Update password in database
    session::update_user_passhash(&pool, auth_user.id, &new_hash).await?;

    // Invalidate all other sessions (keep current one active via new tokens)
    session::delete_all_sessions(&pool, auth_user.id).await?;

    // Create a new session for the current device
    let jwt_secret = session::get_jwt_secret(&pool).await?;
    let role = UserRole::from_admin_fields(user.is_admin, user.admin_level);

    let access_token = tokens::create_access_token(auth_user.id, role, &jwt_secret)?;
    let refresh_token = tokens::generate_refresh_token();
    let refresh_hash = tokens::hash_refresh_token(&refresh_token);

    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let ip_address = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    session::create_session(
        &pool,
        auth_user.id,
        &refresh_hash,
        user_agent.as_deref(),
        ip_address.as_deref(),
    )
    .await?;

    let mut response = HttpResponse::Ok().json(AuthResponse {
        success: true,
        message: Some("Password changed successfully. All other sessions have been invalidated.".to_string()),
        user: None,
    });

    cookies::set_auth_cookies(&mut response, &access_token, &refresh_token);

    Ok(response)
}

// ============================================================
// Request password reset
// ============================================================

pub async fn request_password_reset(
    pool: web::Data<DbPool>,
    body: web::Json<PasswordResetRequest>,
) -> Result<HttpResponse, AuthError> {
    // Always return success to prevent email enumeration
    let success_response = HttpResponse::Ok().json(AuthResponse {
        success: true,
        message: Some("If an account with that email exists, a password reset link has been sent.".to_string()),
        user: None,
    });

    // Look up user by email
    let user = match session::find_user_by_email(&pool, &body.email).await? {
        Some(u) => u,
        None => return Ok(success_response), // Don't reveal if email exists
    };

    // Generate token and store hash
    let raw_token = tokens::generate_random_token();
    let token_hash = tokens::hash_refresh_token(&raw_token);

    session::create_password_reset(&pool, user.id, &token_hash).await?;

    // In production, send an email. For now, log the token.
    tracing::info!(
        "Password reset token generated for user {} (id: {}). Token: {}",
        user.name, user.id, raw_token
    );

    Ok(success_response)
}

// ============================================================
// Complete password reset
// ============================================================

pub async fn complete_password_reset(
    pool: web::Data<DbPool>,
    body: web::Json<PasswordResetComplete>,
) -> Result<HttpResponse, AuthError> {
    // Validate new password
    password::validate_password_length(&body.new_password)?;

    // Hash the submitted token for comparison
    let submitted_hash = tokens::hash_refresh_token(&body.token);

    // We need to find which user this token belongs to.
    // Search all valid (non-expired, non-used) resets.
    // This is a brute-force approach; in production with many users,
    // you'd want to include user_id or email in the reset request.
    let conn = &mut pool.get().await.map_err(|e| {
        AuthError::DatabaseError(format!("Pool error: {}", e))
    })?;

    #[derive(diesel::QueryableByName)]
    struct ResetMatch {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        user_id: Uuid,
    }

    use diesel::sql_query;
    use diesel::sql_types::Text;
    use diesel_async::RunQueryDsl;

    let reset_match: ResetMatch = sql_query(
        "SELECT id, user_id FROM password_resets
         WHERE reset_token = $1 AND used_at IS NULL AND expires_at > NOW()
         LIMIT 1"
    )
    .bind::<Text, _>(&submitted_hash)
    .get_result(conn)
    .await
    .map_err(|_| AuthError::InvalidResetToken)?;

    // Hash new password
    let new_hash = password::hash_password(&body.new_password)?;

    // Update user password
    session::update_user_passhash(&pool, reset_match.user_id, &new_hash).await?;

    // Mark reset token as used
    session::mark_reset_used(&pool, reset_match.id).await?;

    // Invalidate all sessions for this user
    session::delete_all_sessions(&pool, reset_match.user_id).await?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        success: true,
        message: Some("Password has been reset. Please log in with your new password.".to_string()),
        user: None,
    }))
}

// ============================================================
// Verify email
// ============================================================

pub async fn verify_email(
    pool: web::Data<DbPool>,
    body: web::Json<EmailVerifyRequest>,
) -> Result<HttpResponse, AuthError> {
    let submitted_hash = tokens::hash_refresh_token(&body.token);

    let conn = &mut pool.get().await.map_err(|e| {
        AuthError::DatabaseError(format!("Pool error: {}", e))
    })?;

    #[derive(diesel::QueryableByName)]
    struct VerifyMatch {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        user_id: Uuid,
    }

    use diesel::sql_query;
    use diesel::sql_types::Text;
    use diesel_async::RunQueryDsl;

    let verify_match: VerifyMatch = sql_query(
        "SELECT id, user_id FROM email_verification
         WHERE verification_code = $1 AND verified_at IS NULL
         LIMIT 1"
    )
    .bind::<Text, _>(&submitted_hash)
    .get_result(conn)
    .await
    .map_err(|_| AuthError::InvalidVerificationToken)?;

    // Mark verification as complete
    session::mark_email_verified(&pool, verify_match.id).await?;

    // Update user's email_verified flag
    session::set_email_verified(&pool, verify_match.user_id).await?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        success: true,
        message: Some("Email verified successfully.".to_string()),
        user: None,
    }))
}

// ============================================================
// Request email verification
// ============================================================

pub async fn request_email_verification(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    body: web::Json<RequestEmailVerification>,
) -> Result<HttpResponse, AuthError> {
    let auth_user = req.require_auth()?;

    // Check if already verified
    let user = session::get_user_by_id(&pool, auth_user.id).await?;
    if user.is_email_verified {
        return Err(AuthError::AlreadyVerified);
    }

    // Generate verification token
    let raw_token = tokens::generate_random_token();
    let token_hash = tokens::hash_refresh_token(&raw_token);

    session::create_email_verification(&pool, auth_user.id, &body.email, &token_hash).await?;

    // In production, send an email. For now, log the token.
    tracing::info!(
        "Email verification token generated for user {} (id: {}). Token: {}",
        user.name, auth_user.id, raw_token
    );

    Ok(HttpResponse::Ok().json(AuthResponse {
        success: true,
        message: Some("Verification email has been sent.".to_string()),
        user: None,
    }))
}

/// Configure the auth routes scope with CSRF protection.
pub fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    // The JWT secret is loaded from the database at startup and shared via app_data.
    // The auth middleware reads it from the pool on first use.
    cfg.service(
        web::scope("/api/v2/auth")
            .wrap(CsrfGuard)
            .route("/login", web::post().to(login))
            .route("/register", web::post().to(register))
            .route("/logout", web::post().to(logout))
            .route("/logout-all", web::post().to(logout_all))
            .route("/refresh", web::post().to(refresh_token))
            .route("/change-password", web::post().to(change_password))
            .route("/password-reset/request", web::post().to(request_password_reset))
            .route("/password-reset/complete", web::post().to(complete_password_reset))
            .route("/email/verify", web::post().to(verify_email))
            .route("/email/request-verification", web::post().to(request_email_verification))
    );
}

/// Variant that takes a JWT secret and wraps routes with the AuthMiddleware.
/// This ensures handlers that call `req.require_auth()` have the AuthenticatedUser
/// populated from the tb_access cookie.
pub fn configure_auth_routes_with_secret(jwt_secret: String) -> impl FnOnce(&mut web::ServiceConfig) {
    move |cfg: &mut web::ServiceConfig| {
        cfg.service(
            web::scope("/api/v2/auth")
                .wrap(CsrfGuard)
                .wrap(crate::middleware::AuthMiddleware::new(jwt_secret))
                .route("/login", web::post().to(login))
                .route("/register", web::post().to(register))
                .route("/logout", web::post().to(logout))
                .route("/logout-all", web::post().to(logout_all))
                .route("/refresh", web::post().to(refresh_token))
                .route("/change-password", web::post().to(change_password))
                .route("/password-reset/request", web::post().to(request_password_reset))
                .route("/password-reset/complete", web::post().to(complete_password_reset))
                .route("/email/verify", web::post().to(verify_email))
                .route("/email/request-verification", web::post().to(request_email_verification))
        );
    }
}
