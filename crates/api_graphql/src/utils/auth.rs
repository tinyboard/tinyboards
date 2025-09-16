use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token, VerifyWithKey};
use sha2::Sha384;
use std::collections::BTreeMap;
use tinyboards_db::{
    models::{secret::Secret, user::user::User},
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

/// Generate a JWT token for a user
pub fn get_jwt(uid: i32, uname: &str, master_key: &Secret) -> String {
    let key: Hmac<Sha384> = Hmac::new_from_slice(master_key.jwt.as_bytes()).unwrap();
    let header = Header {
        algorithm: AlgorithmType::Hs384,
        ..Default::default()
    };

    let mut claims = BTreeMap::new();
    claims.insert("uid", uid.to_string());
    claims.insert("uname", uname.to_string());

    let token = Token::new(header, claims)
        .sign_with_key(&key)
        .unwrap()
        .as_str()
        .to_string();

    token
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

    // Reference to the string stored in `auth` skipping the `Bearer ` part
    let token = String::from(&auth[7..]);
    let master_key_str = master_key.jwt.clone();

    // Parse and validate JWT token

    let key: Hmac<Sha384> = Hmac::new_from_slice(master_key_str.as_bytes())
        .map_err(|_| TinyBoardsError::from_message(500, "Invalid JWT key"))?;

    let token: Token<Header, BTreeMap<String, String>, _> = token
        .verify_with_key(&key)
        .map_err(|_| TinyBoardsError::from_message(401, "Invalid or expired token"))?;

    let claims = token.claims();

    // Extract user ID from JWT claims
    let user_id: i32 = claims
        .get("uid")
        .ok_or_else(|| TinyBoardsError::from_message(401, "Invalid token: missing user ID"))?
        .parse()
        .map_err(|_| TinyBoardsError::from_message(401, "Invalid token: invalid user ID"))?;

    // Load user from database
    let user = User::read(pool, user_id).await
        .map_err(|_| TinyBoardsError::from_message(401, "User not found"))?;

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