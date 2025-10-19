use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use tinyboards_db::{
    models::{secret::Secret, user::user::User},
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// User ID
    pub sub: i32,
    /// Username
    pub uname: String,
    /// Issuer (hostname)
    pub iss: String,
    /// Issued at time as UNIX timestamp
    pub iat: i64,
    /// Expiration time as UNIX timestamp (optional)
    pub exp: Option<i64>,
}

/// Generate a JWT token for a user
pub fn get_jwt(uid: i32, uname: &str, master_key: &Secret) -> Result<String, TinyBoardsError> {
    let now = Utc::now().timestamp();

    let claims = Claims {
        sub: uid,
        uname: uname.to_string(),
        iss: "tinyboards".to_string(), // or get from config
        iat: now,
        exp: Some(now + 86400), // 24 hours from now
    };

    let key = EncodingKey::from_secret(master_key.jwt_secret.as_bytes());
    let token = encode(&Header::default(), &claims, &key)
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to create JWT token"))?;

    Ok(token)
}

/// Extract user from Authorization header (Bearer token)
pub async fn get_user_from_header_opt(
    pool: &DbPool,
    master_key: &Secret,
    auth: Option<&str>,
) -> Result<Option<User>, TinyBoardsError> {
    if auth.is_none() {
        return Ok(None);
    }

    let auth = auth.unwrap();
    if auth.is_empty() {
        return Ok(None);
    }

    if !auth.starts_with("Bearer ") {
        return Err(TinyBoardsError::from_message(
            400,
            "Invalid `Authorization` header! It should be `Authorization: Bearer <access token>`",
        ));
    }

    // Extract token from "Bearer <token>"
    let token = &auth[7..];

    // Decode and validate JWT token
    let mut validation = Validation::default();
    validation.validate_exp = true; // Enable expiration validation

    let key = DecodingKey::from_secret(master_key.jwt_secret.as_bytes());
    let token_data: TokenData<Claims> = decode(token, &key, &validation)
        .map_err(|_| TinyBoardsError::from_message(401, "Invalid or expired token"))?;

    let claims = token_data.claims;

    // Load user from database
    let user = User::read(pool, claims.sub).await
        .map_err(|_| TinyBoardsError::from_message(401, "User not found"))?;

    // Update last_seen timestamp (fire and forget - don't block on this)
    let pool_clone = pool.clone();
    let user_id = user.id;
    tokio::spawn(async move {
        use tinyboards_db::models::user::user::UserForm;
        use tinyboards_db::traits::Crud;
        use chrono::Utc;

        let update_form = UserForm {
            last_seen: Some(Utc::now().naive_utc()),
            ..UserForm::default()
        };

        let _ = User::update(&pool_clone, user_id, &update_form).await;
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