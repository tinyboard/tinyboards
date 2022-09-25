use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct OAuthApps {
    id: i32,
    client_id: String,
    client_secret: String,
    app_name: String,
    redirect_uri: String,
    author_id: i32,
    is_banned: Bool,
    app_description: String,
}