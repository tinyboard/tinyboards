use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;
use std::task::{Context, Poll};

use crate::cookies::{ACCESS_COOKIE_NAME, clear_access_cookie};
use crate::tokens::validate_access_token;
use crate::types::AuthenticatedUser;

/// Actix middleware that extracts the authenticated user from the `tb_access` cookie.
///
/// On every request:
/// 1. Reads the `tb_access` cookie
/// 2. If absent, continues as anonymous (no user in request extensions)
/// 3. If present, validates the JWT
/// 4. If valid, attaches `AuthenticatedUser` to request extensions
/// 5. If invalid/expired, clears the bad cookie and continues as anonymous
pub struct AuthMiddleware {
    jwt_secret: String,
}

impl AuthMiddleware {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Rc::new(service),
            jwt_secret: self.jwt_secret.clone(),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    jwt_secret: String,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
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
        let jwt_secret = self.jwt_secret.clone();

        Box::pin(async move {
            // 1. Extract tb_access cookie
            let access_cookie = req.cookie(ACCESS_COOKIE_NAME);

            match access_cookie {
                None => {
                    // No cookie — continue as anonymous
                    service.call(req).await
                }
                Some(cookie) => {
                    // 2. Validate JWT
                    match validate_access_token(cookie.value(), &jwt_secret) {
                        Ok(claims) => {
                            // 3. Attach authenticated user to request extensions
                            req.extensions_mut().insert(AuthenticatedUser {
                                id: claims.sub,
                                role: claims.role,
                            });
                            service.call(req).await
                        }
                        Err(_) => {
                            // Invalid/expired token — continue as anonymous, clear the bad cookie
                            tracing::debug!("Invalid access token in cookie, clearing");
                            let mut res = service.call(req).await?;
                            res.response_mut().add_cookie(&clear_access_cookie()).ok();
                            Ok(res)
                        }
                    }
                }
            }
        })
    }
}

/// Extension trait for easily extracting the authenticated user from a request.
pub trait AuthExt {
    /// Get the authenticated user if present.
    fn auth_user(&self) -> Option<AuthenticatedUser>;

    /// Require authentication; returns the user or an AuthError.
    fn require_auth(&self) -> Result<AuthenticatedUser, crate::errors::AuthError>;
}

impl AuthExt for actix_web::HttpRequest {
    fn auth_user(&self) -> Option<AuthenticatedUser> {
        HttpMessage::extensions(self).get::<AuthenticatedUser>().cloned()
    }

    fn require_auth(&self) -> Result<AuthenticatedUser, crate::errors::AuthError> {
        self.auth_user().ok_or(crate::errors::AuthError::LoginRequired)
    }
}

/// Helper to set the RLS variable for the current database connection.
///
/// Call this when creating the GraphQL context for authenticated requests.
pub async fn set_rls_user(
    conn: &mut diesel_async::AsyncPgConnection,
    user_id: uuid::Uuid,
) -> Result<(), crate::errors::AuthError> {
    use diesel::sql_query;
    use diesel::sql_types::Text;
    use diesel_async::RunQueryDsl;

    sql_query("SET LOCAL app.current_user_id = $1")
        .bind::<Text, _>(user_id.to_string())
        .execute(conn)
        .await
        .map_err(|e| crate::errors::AuthError::DatabaseError(
            format!("Failed to set RLS user: {}", e)
        ))?;
    Ok(())
}
