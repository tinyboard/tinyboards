use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct ClientAuths {
    id: i32,
    oauth_client: i32,
    oauth_code: String,
    user_id: i32,
    scope_identity: bool,
    scope_create: bool,
    scope_read: bool,
    scope_update: bool,
    scope_delete: bool,
    scope_vote: bool,
    scope_moderator: bool,
    access_token: String,
    refresh_token: String,
    access_token_expire_utc: i64,
}
