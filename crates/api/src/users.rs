// external crates
use serde::{Deserialize, Serialize};
use regex::Regex;
use jwt::{AlgorithmType, Header, SignWithKey, Token};
use hmac::{Hmac, Mac};
use sha2::Sha384;
use std::collections::BTreeMap;

// internal crates
use crate::data::PorplContext;
use porpl_db::models::user::User;
use porpl_utils::{PorplError, passhash};
use crate::Perform;
use crate::utils::blocking;

#[derive(Deserialize)]
pub struct GetUsers {
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct GetUsersResponse {
    listing: Vec<User>,
}

#[derive(Serialize)]
pub struct GetInsertUserToDbResponse {
    rows_returned: Option<usize>,
}

#[async_trait::async_trait]
impl Perform for GetUsers {
    type Response = GetUsersResponse;

    async fn perform(
        self,
        context: &PorplContext,
        _: Option<&str>,
    ) -> Result<Self::Response, PorplError> {
        let data: &GetUsers = &self;

        let limit = data.limit.unwrap_or(25);

        let users = blocking(context.pool(), move |conn| User::load(conn, limit)).await??;

        Ok(GetUsersResponse { listing: users })
    }
}

#[derive(Deserialize, Debug)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

#[derive(Serialize)]
pub struct CreateUserResponse {
    pub message: String
}

#[derive(Deserialize, Debug)]
pub struct UserLogin {
    pub username: String,
    pub password: String
}

#[derive(Serialize)]
pub struct UserLoginResponse {
    pub message: String,
    pub token: String
}

fn validate_username(username: &str) -> Result<(), PorplError> {
    let re = Regex::new(r"^[A-Za-z][A-Za-z0-9_]{2,29}$").unwrap();
    if re.is_match(username) {
        Ok(())
    } else {
        Err(PorplError::new(400, String::from("Invalid username!")))
    }
}

fn generate_user_jwt(uid: &i32, login_nonce: &i64) -> String {

    let master_secret = std::env::var("MASTER_KEY").unwrap();
    let key:Hmac<Sha384> = Hmac::new_from_slice(master_secret.as_bytes()).unwrap();
    let header = Header {
        algorithm: AlgorithmType::Hs384,
        ..Default::default()
    };


    let mut claims = BTreeMap::new();
    claims.insert("uid", uid.to_string());
    claims.insert("nonce", login_nonce.to_string());

    let token = Token::new(header, claims)
        .sign_with_key(&key)
        .unwrap()
        .as_str()
        .to_string();

    token
}

#[async_trait::async_trait] 
impl Perform for CreateUser {
    type Response = CreateUserResponse;

    async fn perform(
        self,
        context: &PorplContext,
        _: Option<&str>,
    ) -> Result<Self::Response, PorplError> {
        let data: CreateUser = self;

        validate_username(&data.username)?;

        let _new_user = blocking(context.pool(), move |conn| {
            User::insert(conn, data.username, data.password, data.email)
        })
        .await??;

        Ok(CreateUserResponse {
            message: String::from("User created!"),
        })
    }
}


#[async_trait::async_trait]
impl Perform for UserLogin {
    
    type Response = UserLoginResponse;

    async fn perform(
        self,
        context: &PorplContext,
        _: Option<&str>,
    ) -> Result<Self::Response, PorplError> {
        
        let data: UserLogin = self;

        let login_details = blocking(context.pool(), move |conn| {
            User::get_login_details(conn, data.username)
        })
        .await??;

        let uid = login_details.0;
        let hash = login_details.1;
        let login_nonce = login_details.2;

        let new_login_nonce = porpl_utils::time::utc_timestamp();

        blocking(context.pool(), move |conn| {
            User::update_login_nonce(conn, uid, new_login_nonce)
        })
        .await??;

        match passhash::verify_password(&hash, &data.password) {
            true => Ok(UserLoginResponse {
                        message: String::from("Login Successful!"),
                        token: generate_user_jwt(&uid, &login_nonce),
                     }),
            _ => Err(PorplError::new(400, String::from("Invalid password, Login Failed!")))
        }
    }

}


#[test]
fn test_validate_username() {
    assert!(validate_username("   a silly little username ").is_err());
    assert!(validate_username("!2~`23132`Acs*9").is_err());
    assert!(validate_username("perfectlyValidUser").is_ok());
}