use crate::models::users::{InsertUser, User, UserForm};
use crate::traits::Crud;
use diesel::prelude::*;
use diesel::PgConnection;
use porpl_utils::PorplError;

use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token, VerifyWithKey};
use sha2::Sha384;
use std::collections::BTreeMap;

impl User {
    /// Checks if an account with specified username/email already exists.
    fn check_reserved(
        conn: &mut PgConnection,
        name: &str,
        email_addr: &Option<String>,
    ) -> Result<(), PorplError> {
        use crate::schema::users::dsl::*;

        // this feels too repetitive, maybe I'll do something with it someday
        let user = if let Some(email_addr) = email_addr {
            // if an email address is provided, add an extra check for whether it's already taken
            users
                .select(id)
                .filter(username.ilike(name))
                .or_filter(email.ilike(email_addr))
                .first::<i32>(conn)
        } else {
            // else check for username only
            users
                .select(id)
                .filter(username.ilike(name))
                .first::<i32>(conn)
        }
        .optional()
        .map_err(|e| {
            eprintln!("ERROR: {}", e);
            PorplError::err_500()
        })?;

        // if the query above has returned a record, the name/email is already taken; throw error
        if user.is_some() {
            return Err(PorplError::new(
                409,
                String::from("Username/email already taken!"),
            ));
        }

        Ok(())
    }

    pub fn get_jwt(uid: i32, login_nonce: i64, master_key: &str) -> String {
        let key: Hmac<Sha384> = Hmac::new_from_slice(master_key.as_bytes()).unwrap();
        let header = Header {
            algorithm: AlgorithmType::Hs384,
            ..Default::default()
        };

        let mut claims = BTreeMap::new();
        claims.insert("uid", uid.to_string());
        claims.insert("login_nonce", login_nonce.to_string());

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
        master_key: String,
    ) -> Result<Option<Self>, PorplError> {
        use crate::schema::users::dsl::*;

        let key: Hmac<Sha384> = Hmac::new_from_slice(master_key.as_bytes()).unwrap();
        let claims: BTreeMap<String, String> = token.verify_with_key(&key).map_err(|e| {
            eprintln!("ERROR: {:#?}", e);
            PorplError::err_500()
        })?;

        let uid = claims["uid"]
            .parse::<i32>()
            .map_err(|_| PorplError::err_500())?;
        let u_login_nonce = claims["login_nonce"]
            .parse::<i64>()
            .map_err(|_| PorplError::err_500())?;

        users
            .filter(id.eq(uid))
            .filter(login_nonce.eq(u_login_nonce))
            .first::<Self>(conn)
            .optional()
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
    }

    /**
     * load up to a `limit` amount of users. Requires a **mutable** reference to a database connection.
     */
    pub fn load(conn: &mut PgConnection, limit: i64) -> Result<Vec<Self>, PorplError> {
        use crate::schema::users::dsl::*;
        users.limit(limit).load::<Self>(conn).map_err(|e| {
            eprintln!("ERROR: {}", e);
            PorplError::err_500()
        })
    }

    pub fn get_login_details(
        conn: &mut PgConnection,
        name: String,
    ) -> Result<(i32, String, i64), PorplError> {
        use crate::schema::users::dsl::*;

        // escape wildcards (prevent abooz)
        let name = name.replace('%', "\\%").replace('_', "\\_");
        let result = users
            .select((id, passhash, login_nonce))
            .filter(username.ilike(name))
            .first::<(i32, String, i64)>(conn)
            .map_err(|_| {
                PorplError::new(
                    404,
                    String::from(
                        "There's no account with that username. Consider signing up instead?",
                    ),
                )
            })?;
        Ok(result)
    }

    pub fn update_login_nonce(
        conn: &mut PgConnection,
        uid: i32,
        nonce: i64,
    ) -> Result<usize, PorplError> {
        use crate::schema::users::dsl::*;

        let result = diesel::update(users)
            .filter(id.eq(uid))
            .set(login_nonce.eq(nonce))
            .execute(conn)
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
            .unwrap();

        Ok(result)
    }

    pub fn insert(
        conn: &mut PgConnection,
        username: String,
        password: String,
        email: Option<String>,
    ) -> Result<Self, PorplError> {
        use crate::schema::users;

        // escape wildcards
        let username = username.replace('%', "\\%").replace('_', "\\_");
        let email: Option<String> =
            email.map(|email| email.replace('%', "\\%").replace('_', "\\_"));

        Self::check_reserved(conn, &username, &email)?;

        let new_user = InsertUser {
            username,
            passhash: porpl_utils::hash_password(password),
            email,
            created_utc: 12,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result(conn)
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
    }
}

impl Crud for User {
    type Form = UserForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, user_id: i32) -> Result<Self, PorplError> {
        use crate::schema::users::dsl::*;
        users
            .filter(is_deleted.eq(false))
            .find(user_id)
            .first::<Self>(conn)
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
    }
    fn delete(conn: &mut PgConnection, user_id: i32) -> Result<usize, PorplError> {
        use crate::schema::users::dsl::*;
        diesel::delete(users.find(user_id))
            .execute(conn)
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
    }
    fn create(conn: &mut PgConnection, form: &UserForm) -> Result<Self, PorplError> {
        use crate::schema::users::dsl::*;
        diesel::insert_into(users)
            .values(form)
            .get_result::<Self>(conn)
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
    }
    fn update(conn: &mut PgConnection, user_id: i32, form: &UserForm) -> Result<Self, PorplError> {
        use crate::schema::users::dsl::*;
        diesel::update(users.find(user_id))
            .set(form)
            .get_result::<Self>(conn)
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
    }
}
