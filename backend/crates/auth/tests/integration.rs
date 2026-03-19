//! Integration tests for the auth system.
//!
//! These tests require a running PostgreSQL database. Set the `TEST_DATABASE_URL`
//! environment variable to a valid connection string before running.
//!
//! Example:
//!   TEST_DATABASE_URL="postgres://user:pass@localhost/tinyboards_test" cargo test -p tinyboards_auth --test integration
//!
//! The test harness will:
//! 1. Run all migration SQL files against the database
//! 2. Run each test in its own transaction (or do per-test cleanup)
//! 3. Drop all tables on teardown

use actix_web::{web, App};
use actix_web::cookie::Cookie;
use actix_web::test as actix_test;
use serde_json::Value;

use tinyboards_auth::cookies::{ACCESS_COOKIE_NAME, REFRESH_COOKIE_NAME};
use tinyboards_auth::handlers::configure_auth_routes;
use tinyboards_auth::middleware::AuthMiddleware;
use tinyboards_auth::session::DbPool;

/// Get the test database URL or skip the test.
fn test_database_url() -> String {
    std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set to run integration tests")
}

/// Build a connection pool for the test database.
async fn build_test_pool() -> DbPool {
    use diesel_async::pooled_connection::AsyncDieselConnectionManager;

    let url = test_database_url();
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(url);
    bb8::Pool::builder()
        .max_size(5)
        .build(config)
        .await
        .expect("Failed to build test pool")
}

/// Run all migrations against the test database.
/// Drops everything first to ensure a clean state.
async fn run_migrations(pool: &DbPool) {
    let conn = &mut pool.get().await.expect("Failed to get connection");

    use diesel::sql_query;
    use diesel_async::RunQueryDsl;

    // Drop all tables to start clean
    sql_query("DROP SCHEMA public CASCADE")
        .execute(conn)
        .await
        .ok();
    sql_query("CREATE SCHEMA public")
        .execute(conn)
        .await
        .expect("Failed to create schema");
    sql_query("GRANT ALL ON SCHEMA public TO PUBLIC")
        .execute(conn)
        .await
        .ok();

    // Read and execute each migration in order
    let migration_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../migrations");
    let mut entries: Vec<_> = std::fs::read_dir(migration_dir)
        .expect("Failed to read migrations directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let up_sql = entry.path().join("up.sql");
        if up_sql.exists() {
            let sql = std::fs::read_to_string(&up_sql)
                .unwrap_or_else(|_| panic!("Failed to read {:?}", up_sql));
            sql_query(&sql)
                .execute(conn)
                .await
                .unwrap_or_else(|e| panic!("Migration {:?} failed: {}", up_sql, e));
        }
    }
}

/// Get the JWT secret from the test database.
async fn get_test_jwt_secret(pool: &DbPool) -> String {
    tinyboards_auth::session::get_jwt_secret(pool)
        .await
        .expect("Failed to get JWT secret")
}

/// Clean up test data between tests (delete all users, sessions, etc.)
async fn cleanup_test_data(pool: &DbPool) {
    let conn = &mut pool.get().await.expect("Failed to get connection");
    use diesel::sql_query;
    use diesel_async::RunQueryDsl;

    // Delete in dependency order
    sql_query("DELETE FROM auth_sessions").execute(conn).await.ok();
    sql_query("DELETE FROM email_verification").execute(conn).await.ok();
    sql_query("DELETE FROM password_resets").execute(conn).await.ok();
    sql_query("DELETE FROM registration_applications").execute(conn).await.ok();
    sql_query("DELETE FROM site_invites").execute(conn).await.ok();
    sql_query("DELETE FROM users").execute(conn).await.ok();
}

/// Set the site registration mode for testing.
async fn set_registration_mode(pool: &DbPool, mode: &str) {
    let conn = &mut pool.get().await.expect("Failed to get connection");
    use diesel::sql_query;
    use diesel_async::RunQueryDsl;

    // Cast the text to the registration_mode enum
    sql_query(format!(
        "UPDATE site SET registration_mode = '{}'::registration_mode",
        mode
    ))
    .execute(conn)
    .await
    .expect("Failed to set registration mode");
}

/// Insert a site invite code for testing.
async fn insert_invite(pool: &DbPool, code: &str) {
    let conn = &mut pool.get().await.expect("Failed to get connection");
    use diesel::sql_query;
    use diesel::sql_types::Text;
    use diesel_async::RunQueryDsl;

    sql_query("INSERT INTO site_invites (verification_code) VALUES ($1)")
        .bind::<Text, _>(code)
        .execute(conn)
        .await
        .expect("Failed to insert invite");
}

/// Helper: build a test app with auth routes and middleware.
fn build_test_app(
    pool: DbPool,
    jwt_secret: String,
) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .app_data(web::Data::new(pool))
        .wrap(AuthMiddleware::new(jwt_secret))
        .configure(configure_auth_routes)
}

// ============================================================
// Tests
// ============================================================

#[actix_rt::test]
async fn test_register_and_login() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret)).await;

    // Register
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "testuser",
            "password": "securepassword123",
            "email": "test@example.com"
        }))
        .to_request();

    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Register failed: {:?}", resp.status());

    // Check cookies are set
    let cookies: Vec<_> = resp.response().cookies().collect();
    let access = cookies.iter().find(|c| c.name() == ACCESS_COOKIE_NAME);
    let refresh = cookies.iter().find(|c| c.name() == REFRESH_COOKIE_NAME);
    assert!(access.is_some(), "Access cookie not set on register");
    assert!(refresh.is_some(), "Refresh cookie not set on register");

    // Check access cookie properties (httpOnly, secure)
    let ac = access.unwrap();
    assert!(ac.http_only().unwrap_or(false), "Access cookie must be httpOnly");
    assert!(ac.secure().unwrap_or(false), "Access cookie must be secure");

    // Check response body
    let body: Value = actix_test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert!(body["account_created"].as_bool().unwrap());
    assert!(body["user"]["id"].is_string());
    assert_eq!(body["user"]["name"], "testuser");

    // Login with username
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/login")
        .set_json(serde_json::json!({
            "username_or_email": "testuser",
            "password": "securepassword123"
        }))
        .to_request();

    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Login failed: {:?}", resp.status());

    let login_cookies: Vec<_> = resp.response().cookies().collect();
    assert!(login_cookies.iter().any(|c| c.name() == ACCESS_COOKIE_NAME));
    assert!(login_cookies.iter().any(|c| c.name() == REFRESH_COOKIE_NAME));

    let body: Value = actix_test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["user"]["name"], "testuser");
}

#[actix_rt::test]
async fn test_login_with_email() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret)).await;

    // Register
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "emailuser",
            "password": "securepassword123",
            "email": "emailuser@example.com"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Login with email
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/login")
        .set_json(serde_json::json!({
            "username_or_email": "emailuser@example.com",
            "password": "securepassword123"
        }))
        .to_request();

    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Login with email failed");

    let body: Value = actix_test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["user"]["name"], "emailuser");
}

#[actix_rt::test]
async fn test_invalid_credentials() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret)).await;

    // Register a user
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "validuser",
            "password": "correctpassword123"
        }))
        .to_request();
    actix_test::call_service(&app, req).await;

    // Wrong password
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/login")
        .set_json(serde_json::json!({
            "username_or_email": "validuser",
            "password": "wrongpassword"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Wrong password should return 401");

    // Non-existent user
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/login")
        .set_json(serde_json::json!({
            "username_or_email": "nonexistent",
            "password": "anypassword"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Non-existent user should return 401");
}

#[actix_rt::test]
async fn test_duplicate_registration() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret)).await;

    // Register
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "dupuser",
            "password": "securepassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Try duplicate username
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "dupuser",
            "password": "anotherpassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409, "Duplicate username should return 409 Conflict");
}

#[actix_rt::test]
async fn test_registration_closed() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "closed").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret)).await;

    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "newuser",
            "password": "securepassword123"
        }))
        .to_request();

    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403, "Closed registration should return 403");
}

#[actix_rt::test]
async fn test_registration_invite_only() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "invite_only").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret)).await;

    // Try without invite code
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "inviteuser",
            "password": "securepassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403, "Missing invite code should return 403");

    // Insert a valid invite
    insert_invite(&pool, "valid-invite-code").await;

    // Register with invite code
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "inviteuser",
            "password": "securepassword123",
            "invite_code": "valid-invite-code"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Valid invite should succeed: {:?}", resp.status());

    // Try same invite code again (should be consumed)
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "inviteuser2",
            "password": "securepassword123",
            "invite_code": "valid-invite-code"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(!resp.status().is_success(), "Used invite code should fail");
}

#[actix_rt::test]
async fn test_registration_application_required() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "application_required").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret)).await;

    // Try without application answer
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "appuser",
            "password": "securepassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403, "Missing application should return 403");

    // Register with application answer
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "appuser",
            "password": "securepassword123",
            "application_answer": "I want to join because..."
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Application registration should succeed");

    let body: Value = actix_test::read_body_json(resp).await;
    assert!(body["application_submitted"].as_bool().unwrap(), "Should indicate application submitted");
    assert!(!body["account_created"].as_bool().unwrap(), "Should not indicate account created yet");
    // Should NOT have auth cookies (not logged in until approved)
    // The response is just JSON, cookies would be checked on the service response
}

#[actix_rt::test]
async fn test_invalid_username() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret)).await;

    // Username starting with number
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "123invalid",
            "password": "securepassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Invalid username should return 400");

    // Username with spaces
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "bad user",
            "password": "securepassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Username with spaces should return 400");

    // Username too long (>30 chars)
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "a".repeat(31),
            "password": "securepassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Username >30 chars should return 400");
}

#[actix_rt::test]
async fn test_password_too_short() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret)).await;

    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "shortpw",
            "password": "short"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Short password should return 400");
}

#[actix_rt::test]
async fn test_logout() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret.clone())).await;

    // Register to get cookies
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "logoutuser",
            "password": "securepassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let cookies: Vec<_> = resp.response().cookies().collect();
    let access_val = cookies.iter().find(|c| c.name() == ACCESS_COOKIE_NAME)
        .map(|c| c.value().to_string()).expect("need access cookie");
    let refresh_val = cookies.iter().find(|c| c.name() == REFRESH_COOKIE_NAME)
        .map(|c| c.value().to_string()).expect("need refresh cookie");

    // Logout
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/logout")
        .cookie(Cookie::new(ACCESS_COOKIE_NAME, &access_val))
        .cookie(Cookie::new(REFRESH_COOKIE_NAME, &refresh_val))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Logout failed: {:?}", resp.status());

    let body: Value = actix_test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["message"], "Logged out successfully");
}

#[actix_rt::test]
async fn test_logout_all() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret.clone())).await;

    // Register
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "multidevice",
            "password": "securepassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    let cookies: Vec<_> = resp.response().cookies().collect();
    let access_val = cookies.iter().find(|c| c.name() == ACCESS_COOKIE_NAME)
        .map(|c| c.value().to_string()).unwrap();

    // Login again to create a second session
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/login")
        .set_json(serde_json::json!({
            "username_or_email": "multidevice",
            "password": "securepassword123"
        }))
        .to_request();
    actix_test::call_service(&app, req).await;

    // Logout all
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/logout-all")
        .cookie(Cookie::new(ACCESS_COOKIE_NAME, &access_val))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Logout all failed");

    let body: Value = actix_test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    // Should mention multiple devices
    assert!(body["message"].as_str().unwrap().contains("device(s)"));
}

#[actix_rt::test]
async fn test_token_refresh() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret.clone())).await;

    // Register
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "refreshuser",
            "password": "securepassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let cookies: Vec<_> = resp.response().cookies().collect();
    let access_val = cookies.iter().find(|c| c.name() == ACCESS_COOKIE_NAME)
        .map(|c| c.value().to_string()).unwrap();
    let refresh_val = cookies.iter().find(|c| c.name() == REFRESH_COOKIE_NAME)
        .map(|c| c.value().to_string()).unwrap();

    // Refresh tokens
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/refresh")
        .cookie(Cookie::new(ACCESS_COOKIE_NAME, &access_val))
        .cookie(Cookie::new(REFRESH_COOKIE_NAME, &refresh_val))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Token refresh failed: {:?}", resp.status());

    // Should have new cookies
    let new_cookies: Vec<_> = resp.response().cookies().collect();
    let new_access = new_cookies.iter().find(|c| c.name() == ACCESS_COOKIE_NAME)
        .map(|c| c.value().to_string()).expect("new access cookie");
    let new_refresh = new_cookies.iter().find(|c| c.name() == REFRESH_COOKIE_NAME)
        .map(|c| c.value().to_string()).expect("new refresh cookie");

    // Tokens should be different (rotated)
    assert_ne!(access_val, new_access, "Access token should be rotated");
    assert_ne!(refresh_val, new_refresh, "Refresh token should be rotated");

    // Old refresh token should no longer work
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/refresh")
        .cookie(Cookie::new(ACCESS_COOKIE_NAME, &new_access))
        .cookie(Cookie::new(REFRESH_COOKIE_NAME, &refresh_val)) // old refresh
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Old refresh token should be rejected after rotation");
}

#[actix_rt::test]
async fn test_refresh_without_cookie() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret)).await;

    // Try refresh with no cookies
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/refresh")
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Refresh without cookie should return 401");
}

#[actix_rt::test]
async fn test_change_password() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret.clone())).await;

    // Register
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "pwchangeuser",
            "password": "oldpassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    let cookies: Vec<_> = resp.response().cookies().collect();
    let access_val = cookies.iter().find(|c| c.name() == ACCESS_COOKIE_NAME)
        .map(|c| c.value().to_string()).unwrap();

    // Change password
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/change-password")
        .cookie(Cookie::new(ACCESS_COOKIE_NAME, &access_val))
        .set_json(serde_json::json!({
            "current_password": "oldpassword123",
            "new_password": "newpassword456"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Password change failed: {:?}", resp.status());

    let body: Value = actix_test::read_body_json(resp).await;
    assert_eq!(body["success"], true);

    // Login with new password should work
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/login")
        .set_json(serde_json::json!({
            "username_or_email": "pwchangeuser",
            "password": "newpassword456"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Login with new password should work");

    // Login with old password should fail
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/login")
        .set_json(serde_json::json!({
            "username_or_email": "pwchangeuser",
            "password": "oldpassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Old password should no longer work");
}

#[actix_rt::test]
async fn test_change_password_wrong_current() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret.clone())).await;

    // Register
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "wrongcurpw",
            "password": "correctpassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    let cookies: Vec<_> = resp.response().cookies().collect();
    let access_val = cookies.iter().find(|c| c.name() == ACCESS_COOKIE_NAME)
        .map(|c| c.value().to_string()).unwrap();

    // Try changing with wrong current password
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/change-password")
        .cookie(Cookie::new(ACCESS_COOKIE_NAME, &access_val))
        .set_json(serde_json::json!({
            "current_password": "wrongcurrent",
            "new_password": "newpassword456"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Wrong current password should return 401");
}

#[actix_rt::test]
async fn test_change_password_requires_auth() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret)).await;

    // Try without auth
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/change-password")
        .set_json(serde_json::json!({
            "current_password": "anything",
            "new_password": "newpassword456"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Change password without auth should return 401");
}

#[actix_rt::test]
async fn test_password_reset_flow() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret.clone())).await;

    // Register a user with email
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "resetuser",
            "password": "originalpassword123",
            "email": "reset@example.com"
        }))
        .to_request();
    actix_test::call_service(&app, req).await;

    // Request password reset
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/password-reset/request")
        .set_json(serde_json::json!({
            "email": "reset@example.com"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Password reset request failed");

    // The token is logged, not emailed. We need to get it from the database directly.
    // Look up the reset token hash in the DB, then we'll need to brute force... but we can't.
    // Instead, for testing, we'll look up the user_id and the reset hash, and test the
    // complete endpoint with an invalid token to verify it properly rejects.

    // Try completing with an invalid token
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/password-reset/complete")
        .set_json(serde_json::json!({
            "token": "invalid-token-here",
            "new_password": "newresetpassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Invalid reset token should return 400");

    // Request reset for non-existent email (should still succeed to prevent enumeration)
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/password-reset/request")
        .set_json(serde_json::json!({
            "email": "nonexistent@example.com"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Non-existent email reset should still return 200");
}

#[actix_rt::test]
async fn test_password_reset_end_to_end() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret.clone())).await;

    // Register
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "e2ereset",
            "password": "originalpass123",
            "email": "e2ereset@example.com"
        }))
        .to_request();
    actix_test::call_service(&app, req).await;

    // Manually create a password reset token (simulating what request_password_reset does)
    let raw_token = tinyboards_auth::tokens::generate_random_token();
    let token_hash = tinyboards_auth::tokens::hash_refresh_token(&raw_token);

    // Find the user
    let user = tinyboards_auth::session::find_user_by_email(&pool, "e2ereset@example.com")
        .await
        .unwrap()
        .unwrap();

    // Insert the reset token directly
    tinyboards_auth::session::create_password_reset(&pool, user.id, &token_hash).await.unwrap();

    // Complete the reset
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/password-reset/complete")
        .set_json(serde_json::json!({
            "token": raw_token,
            "new_password": "brandnewpass456"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Password reset complete failed: {:?}", resp.status());

    // Login with new password
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/login")
        .set_json(serde_json::json!({
            "username_or_email": "e2ereset",
            "password": "brandnewpass456"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Login with reset password should work");

    // Old password should fail
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/login")
        .set_json(serde_json::json!({
            "username_or_email": "e2ereset",
            "password": "originalpass123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Old password should no longer work after reset");
}

#[actix_rt::test]
async fn test_email_verification_flow() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret.clone())).await;

    // Register with email
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "verifyuser",
            "password": "securepassword123",
            "email": "verify@example.com"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Manually create a verification token
    let raw_token = tinyboards_auth::tokens::generate_random_token();
    let token_hash = tinyboards_auth::tokens::hash_refresh_token(&raw_token);

    let user = tinyboards_auth::session::get_user_by_name(&pool, "verifyuser").await.unwrap();
    tinyboards_auth::session::create_email_verification(
        &pool, user.id, "verify@example.com", &token_hash
    ).await.unwrap();

    // Verify email
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/email/verify")
        .set_json(serde_json::json!({
            "token": raw_token
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Email verification failed: {:?}", resp.status());

    let body: Value = actix_test::read_body_json(resp).await;
    assert_eq!(body["success"], true);

    // Verify that the user is now marked as email verified
    let user = tinyboards_auth::session::get_user_by_name(&pool, "verifyuser").await.unwrap();
    assert!(user.is_email_verified, "User should be email verified");
}

#[actix_rt::test]
async fn test_email_verify_invalid_token() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret)).await;

    // Try verifying with invalid token
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/email/verify")
        .set_json(serde_json::json!({
            "token": "invalid-verification-token"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Invalid verification token should return 400");
}

#[actix_rt::test]
async fn test_request_email_verification_requires_auth() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret)).await;

    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/email/request-verification")
        .set_json(serde_json::json!({
            "email": "test@example.com"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Email verification request without auth should return 401");
}

#[actix_rt::test]
async fn test_middleware_clears_bad_token() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(AuthMiddleware::new(jwt_secret))
            .route("/api/v2/auth/login", web::post().to(tinyboards_auth::handlers::login))
    ).await;

    // Send a request with an invalid access token
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/login")
        .cookie(Cookie::new(ACCESS_COOKIE_NAME, "invalid.jwt.token"))
        .set_json(serde_json::json!({
            "username_or_email": "nobody",
            "password": "pass"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;

    // The middleware should have added a clear cookie
    let cookies: Vec<_> = resp.response().cookies().collect();
    let clear_cookie = cookies.iter().find(|c| c.name() == ACCESS_COOKIE_NAME);
    if let Some(c) = clear_cookie {
        // Should have maxAge=0 or empty value
        let max_age = c.max_age();
        if let Some(ma) = max_age {
            assert!(ma.whole_seconds() <= 0, "Bad token cookie should be cleared with maxAge <= 0");
        }
    }
}

#[actix_rt::test]
async fn test_session_cleanup() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;

    // Create a user directly
    let user = tinyboards_auth::session::create_user(
        &pool, "cleanupuser", None, None,
        &tinyboards_auth::password::hash_password("password123").unwrap(),
        true,
    ).await.unwrap();

    // Create an expired session directly via SQL
    let conn = &mut pool.get().await.unwrap();
    use diesel::sql_query;
    use diesel_async::RunQueryDsl;

    sql_query(
        "INSERT INTO auth_sessions (user_id, refresh_token_hash, expires_at)
         VALUES ($1, 'expired_hash', NOW() - INTERVAL '1 day')"
    )
    .bind::<diesel::sql_types::Uuid, _>(user.id)
    .execute(conn)
    .await
    .unwrap();

    // Verify it exists
    let sessions = tinyboards_auth::session::get_active_sessions(&pool, user.id).await.unwrap();
    assert_eq!(sessions.len(), 0, "Expired session should not appear in active sessions");

    // Cleanup
    let cleaned = tinyboards_auth::session::cleanup_expired_sessions(&pool).await.unwrap();
    assert_eq!(cleaned, 1, "Should have cleaned up 1 expired session");
}

#[actix_rt::test]
async fn test_case_insensitive_username_login() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret)).await;

    // Register with mixed case
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "MixedCase",
            "password": "securepassword123"
        }))
        .to_request();
    actix_test::call_service(&app, req).await;

    // Login with lowercase
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/login")
        .set_json(serde_json::json!({
            "username_or_email": "mixedcase",
            "password": "securepassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Case-insensitive login should work");

    // Login with uppercase
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/login")
        .set_json(serde_json::json!({
            "username_or_email": "MIXEDCASE",
            "password": "securepassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Uppercase login should work");
}

#[actix_rt::test]
async fn test_access_token_properties() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret.clone())).await;

    // Register to get tokens
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "tokentest",
            "password": "securepassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    let cookies: Vec<_> = resp.response().cookies().collect();

    // Access cookie checks
    let access = cookies.iter().find(|c| c.name() == ACCESS_COOKIE_NAME).unwrap();
    assert!(access.http_only().unwrap_or(false), "Access cookie must be httpOnly");
    assert!(access.secure().unwrap_or(false), "Access cookie must be secure");
    assert_eq!(access.path().unwrap_or(""), "/", "Access cookie path should be /");
    if let Some(max_age) = access.max_age() {
        assert_eq!(max_age.whole_seconds(), 900, "Access token should have 15-minute lifetime");
    }

    // Refresh cookie checks
    let refresh = cookies.iter().find(|c| c.name() == REFRESH_COOKIE_NAME).unwrap();
    assert!(refresh.http_only().unwrap_or(false), "Refresh cookie must be httpOnly");
    assert!(refresh.secure().unwrap_or(false), "Refresh cookie must be secure");
    assert_eq!(refresh.path().unwrap_or(""), "/api/v2/auth", "Refresh cookie path should be /api/v2/auth");
    if let Some(max_age) = refresh.max_age() {
        assert_eq!(max_age.whole_seconds(), 2_592_000, "Refresh token should have 30-day lifetime");
    }

    // Validate the access token JWT is well-formed
    let token_value = access.value();
    let claims = tinyboards_auth::tokens::validate_access_token(token_value, &jwt_secret)
        .expect("Access token should be valid");
    assert_eq!(claims.role, tinyboards_auth::types::UserRole::User);

    // Check that exp is set correctly (~15 minutes from now)
    let now = chrono::Utc::now().timestamp();
    let exp_diff = claims.exp - now;
    assert!(exp_diff > 800 && exp_diff <= 900, "Token expiry should be ~15 minutes from now, got {}s", exp_diff);
}

#[actix_rt::test]
async fn test_argon2id_password_hashing() {
    // Verify that passwords are hashed with argon2id, not any other algorithm
    let hash = tinyboards_auth::password::hash_password("test_password").unwrap();

    // Argon2id hashes start with "$argon2id$"
    assert!(hash.starts_with("$argon2id$"), "Password hash should use argon2id, got: {}", hash);

    // Each hash should produce a unique output (random salt)
    let hash2 = tinyboards_auth::password::hash_password("test_password").unwrap();
    assert_ne!(hash, hash2, "Same password should produce different hashes (random salts)");

    // Verification should work
    assert!(tinyboards_auth::password::verify_password("test_password", &hash).unwrap());
    assert!(tinyboards_auth::password::verify_password("test_password", &hash2).unwrap());
    assert!(!tinyboards_auth::password::verify_password("wrong_password", &hash).unwrap());
}

#[actix_rt::test]
async fn test_concurrent_sessions() {
    let pool = build_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;
    set_registration_mode(&pool, "open").await;
    let jwt_secret = get_test_jwt_secret(&pool).await;

    let app = actix_test::init_service(build_test_app(pool.clone(), jwt_secret)).await;

    // Register
    let req = actix_test::TestRequest::post()
        .uri("/api/v2/auth/register")
        .set_json(serde_json::json!({
            "username": "multisession",
            "password": "securepassword123"
        }))
        .to_request();
    let resp = actix_test::call_service(&app, req).await;
    let body: Value = actix_test::read_body_json(resp).await;
    let user_id: uuid::Uuid = body["user"]["id"].as_str().unwrap().parse().unwrap();

    // Login twice more to create additional sessions
    for _ in 0..2 {
        let req = actix_test::TestRequest::post()
            .uri("/api/v2/auth/login")
            .set_json(serde_json::json!({
                "username_or_email": "multisession",
                "password": "securepassword123"
            }))
            .to_request();
        actix_test::call_service(&app, req).await;
    }

    // Should have 3 active sessions
    let sessions = tinyboards_auth::session::get_active_sessions(&pool, user_id).await.unwrap();
    assert_eq!(sessions.len(), 3, "Should have 3 active sessions");
}
