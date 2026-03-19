//! Authentication module for TinyBoards.
//!
//! Implements the dual-token auth system:
//! - Short-lived JWT access tokens (15 min) in httpOnly cookies
//! - Long-lived refresh tokens (30 days) stored as hashes in auth_sessions
//!
//! This is a standalone crate that uses raw SQL queries via diesel::sql_query()
//! to decouple from the tinyboards_db model layer.

pub mod claims;
pub mod cookies;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod password;
pub mod session;
pub mod tokens;
pub mod types;

// Re-export key types for convenience
pub use errors::AuthError;
pub use handlers::{configure_auth_routes, configure_auth_routes_with_secret};
pub use middleware::{AuthMiddleware, AuthExt};
pub use session::DbPool;
pub use types::{AuthenticatedUser, UserRole};
