use crate::error::TinyBoardsError;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};

type Jwt = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// user_id, standard claim.
    pub sub: i32,
    pub iss: String,
    /// Time when this tokenn was issued as a UNIX-timestamp in seconds
    pub iat: i64,
}

impl Claims {
    pub fn decode(jwt: &str, jwt_secret: &str) -> Result<TokenData<Claims>, TinyBoardsError> {
        let mut validation = Validation::default();
        validation.validate_exp = false;
        validation.required_spec_claims.remove("exp");
        let key = DecodingKey::from_secret(jwt_secret.as_ref());
        Ok(decode::<Claims>(jwt, &key, &validation)?)
    }

    pub fn jwt(user_id: i32, jwt_secret: &str, hostname: &str) -> Result<Jwt, TinyBoardsError> {
        let claims = Claims {
            sub: user_id,
            iss: hostname.to_string(),
            iat: Utc::now().timestamp(),
        };
        let key = EncodingKey::from_secret(jwt_secret.as_ref());
        Ok(encode(&Header::default(), &claims, &key)?)
    }
}