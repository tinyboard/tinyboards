use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::UserRole;

/// JWT claims for the access token.
///
/// Single implementation — replaces both the old `utils/claims.rs` (BUG-002/BUG-019)
/// and `api/utils/auth.rs` Claims struct.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// User ID (UUID)
    pub sub: Uuid,
    /// User role (serialized as JSON)
    pub role: UserRole,
    /// Issued at (UNIX timestamp)
    pub iat: i64,
    /// Expiration (UNIX timestamp)
    pub exp: i64,
}

impl Claims {
    /// Create claims for a new access token.
    ///
    /// Access tokens are short-lived (15 minutes).
    pub fn new(user_id: Uuid, role: UserRole) -> Self {
        let now = chrono::Utc::now().timestamp();
        Claims {
            sub: user_id,
            role,
            iat: now,
            exp: now + 900, // 15 minutes
        }
    }
}
