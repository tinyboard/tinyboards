use actix_web::HttpResponse;
use actix_web::cookie::{Cookie, SameSite};

/// Access token cookie name.
pub const ACCESS_COOKIE_NAME: &str = "tb_access";

/// Refresh token cookie name.
pub const REFRESH_COOKIE_NAME: &str = "tb_refresh";

/// Path for the refresh token cookie.
/// Set to "/" because the browser talks to the Nuxt BFF at /api/auth/*,
/// not directly to /api/v2/auth/*. A restricted path would prevent the
/// browser from sending the cookie to the BFF proxy.
pub const REFRESH_COOKIE_PATH: &str = "/";

/// Access token cookie lifetime in seconds (7 days).
///
/// The JWT inside the cookie has a 15-minute `exp` claim. This longer cookie
/// lifetime ensures the browser still sends the (expired) cookie to the
/// refresh endpoint, which decodes it with `validate_exp = false` to extract
/// the user ID. Without this, the browser deletes the cookie after 15 minutes
/// and the refresh handler has no way to identify the user.
pub const ACCESS_TOKEN_MAX_AGE: i64 = 604_800;

/// Refresh token lifetime in seconds (30 days).
pub const REFRESH_TOKEN_MAX_AGE: i64 = 2_592_000;

/// Whether to set the Secure flag on auth cookies.
///
/// Reads TLS_ENABLED from the environment at startup. When false (local dev
/// over plain HTTP), cookies omit the Secure flag so browsers will accept them.
fn use_secure_cookies() -> bool {
    std::env::var("TLS_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

/// Build the access token cookie.
///
/// httpOnly, SameSite=Lax, path=/
/// Secure flag is set only when TLS_ENABLED=true.
pub fn build_access_cookie(token: &str) -> Cookie<'static> {
    Cookie::build(ACCESS_COOKIE_NAME, token.to_string())
        .http_only(true)
        .secure(use_secure_cookies())
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(actix_web::cookie::time::Duration::seconds(ACCESS_TOKEN_MAX_AGE))
        .finish()
}

/// Build the refresh token cookie.
///
/// httpOnly, SameSite=Lax, path=/
/// Secure flag is set only when TLS_ENABLED=true.
pub fn build_refresh_cookie(token: &str) -> Cookie<'static> {
    Cookie::build(REFRESH_COOKIE_NAME, token.to_string())
        .http_only(true)
        .secure(use_secure_cookies())
        .same_site(SameSite::Lax)
        .path(REFRESH_COOKIE_PATH)
        .max_age(actix_web::cookie::time::Duration::seconds(REFRESH_TOKEN_MAX_AGE))
        .finish()
}

/// Build a cookie that clears the access token (maxAge=0).
pub fn clear_access_cookie() -> Cookie<'static> {
    Cookie::build(ACCESS_COOKIE_NAME, "")
        .http_only(true)
        .secure(use_secure_cookies())
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(actix_web::cookie::time::Duration::ZERO)
        .finish()
}

/// Build a cookie that clears the refresh token (maxAge=0).
pub fn clear_refresh_cookie() -> Cookie<'static> {
    Cookie::build(REFRESH_COOKIE_NAME, "")
        .http_only(true)
        .secure(use_secure_cookies())
        .same_site(SameSite::Lax)
        .path(REFRESH_COOKIE_PATH)
        .max_age(actix_web::cookie::time::Duration::ZERO)
        .finish()
}

/// Set both auth cookies on an HTTP response.
pub fn set_auth_cookies(response: &mut HttpResponse, access_token: &str, refresh_token: &str) {
    response.add_cookie(&build_access_cookie(access_token)).ok();
    response.add_cookie(&build_refresh_cookie(refresh_token)).ok();
}

/// Clear both auth cookies on an HTTP response.
pub fn clear_auth_cookies(response: &mut HttpResponse) {
    response.add_cookie(&clear_access_cookie()).ok();
    response.add_cookie(&clear_refresh_cookie()).ok();
}
