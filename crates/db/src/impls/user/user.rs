use crate::impls::user;
use crate::models::user::user::{User, InsertUser};
use crate::schema::user_::passhash;
use diesel::prelude::*;
use diesel::PgConnection;
use porpl_utils::PorplError;
use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token, VerifyWithKey};
use sha2::Sha384;
use std::collections::BTreeMap;

impl User {
    fn check_reserved(
        conn: &mut PgConnection,
        uname: &str,
        emailaddr: &Option<&str>,
    ) -> Result<(), PorplError> {
        use crate::schema::user_::dsl::*;

        let user = if let Some(emailaddr) = emailaddr {
            user_
                .select(id)
                .filter(name.ilike(uname))
                .or_filter(email.ilike(emailaddr))
                .first::<i32>(conn)
        } else {
            user_
                .select(id)
                .filter(name.ilike(uname))
                .first::<i32>(conn)
        }
        .optional()
        .map_err(|e| {
            eprintln!("ERROR: {e}");
            PorplError::err_500()
        })?;

        if user.is_some() {
            return Err(PorplError::new(
                400,
                String::from("Username/Email already taken!"),
            ));
        }

        Ok(())
    }

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

    // commenting this for now until we can fix the error with it

    // pub fn from_jwt(
    //     conn: &mut PgConnection,
    //     token: String,
    //     master_key: String
    // ) -> Result<Option<Self>, PorplError> {
    //     use crate::schema::user_::dsl::*;
        
    //     let key: Hmac<Sha384> = Hmac::new_from_slice(master_key.as_bytes()).unwrap();
    //     let claims: BTreeMap<String, String> = token.verify_with_key(&key).map_err(|e| {
    //         eprintln!("ERROR: {:#?}", e);
    //         PorplError::err_500()
    //     })?;

    //     let uid = claims["uid"]
    //         .parse::<i32>()
    //         .map_err(|_| PorplError::err_500())?;
        
    //     let uname = claims["uname"];

    //     user_
    //         .filter(id.eq(uid))
    //         .filter(name.eq(uname))
    //         .first::<Self>(conn)
    //         .optional()
    //         .map_err(|e| {
    //             eprintln!("ERROR: {}", e);
    //             PorplError::err_500()
    //         })

    // }

    pub fn insert(
        conn: &mut PgConnection,
        username: String,
        fedi_name: String,
        password: String,
        email: Option<String>,
    ) -> Result<Self, PorplError> {
        use crate::schema::user_;

        let username = username.replace('%', "\\%").replace('_', "\\_");

        let email: Option<String> =
            email.map(|email| email.replace('%', "\\%").replace('_', "\\_"));
        
        
        let hash = porpl_utils::hash_password(password);
        // TODO - fix below
        
        // Self::check_reserved(conn, &username, &&email)?;

        let new_user = InsertUser {
            name: username,
            fedi_name,
            email,
            passhash: hash,
        };

        diesel::insert_into(user_::table)
            .values(&new_user)
            .get_result::<Self>(conn)
            .map_err(|e| {
                eprintln!("ERROR: {e}");
                PorplError::err_500()
            })
    }


}