use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token};
use sha2::Sha384;
use std::collections::BTreeMap;
use porpl_utils::error::PorplError;

pub fn get_jwt(uid: i32, uname: &str, master_key: &str) -> String {
    let key: Hmac<Sha384> = Hmac::new_from_slice(master_key.as_bytes()).unwrap();
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

pub fn from_jwt(
    conn: &mut PgConnection,
    token: String,
    master_key: String
) -> Result<Option<Self>, PorplError> {
    use crate::schema::user_::dsl::*;
    
    let key: Hmac<Sha384> = Hmac::new_from_slice(master_key.as_bytes()).unwrap();
    let claims: BTreeMap<String, String> = token.verify_with_key(&key).map_err(|e| {
        eprintln!("ERROR: {:#?}", e);
        PorplError::err_500()
    })?;

    let uid = claims["uid"]
        .parse::<i32>()
        .map_err(|_| PorplError::err_500())?;
    
    let uname = claims["uname"];

    user_
        .filter(id.eq(uid))
        .filter(name.eq(uname))
        .first::<Self>(conn)
        .optional()
        .map_err(|e| {
            eprintln!("ERROR: {}", e);
            PorplError::err_500()
        })
}