// external crates
use regex::Regex;
use serde::{Deserialize, Serialize};

// internal crates
use crate::data::PorplContext;
use crate::utils::{blocking, require_user};
use crate::Perform;
use porpl_db::models::users::User;
use porpl_utils::{passhash, PorplError};

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

#[derive(Deserialize, Debug)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

#[derive(Serialize)]
pub struct CreateUserResponse {
    pub token: String,
}

#[derive(Deserialize, Debug)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserLoginResponse {
    pub message: String,
    pub token: String,
}

#[derive(Deserialize)]
pub struct GetLoggedInUser {}

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

fn validate_username(username: &str) -> Result<(), PorplError> {
    let re = Regex::new(r"^[A-Za-z][A-Za-z0-9_]{2,29}$").unwrap();
    if re.is_match(username) {
        Ok(())
    } else {
        Err(PorplError::new(400, String::from("Invalid username!")))
    }
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

        let new_user = blocking(context.pool(), move |conn| {
            User::insert(conn, data.username, data.password, data.email)
        })
        .await??;

        Ok(CreateUserResponse {
            token: User::get_jwt(new_user.id, new_user.login_nonce, context.master_key()),
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

        match passhash::verify_password(&hash, &data.password) {
            true => Ok(UserLoginResponse {
                message: String::from("Login Successful!"),
                token: User::get_jwt(uid, login_nonce, context.master_key()),
            }),
            _ => Err(PorplError::new(
                400,
                String::from("Invalid password, Login Failed!"),
            )),
        }
    }
}

#[async_trait::async_trait]
impl Perform for GetLoggedInUser {
    type Response = User;

    async fn perform(
        self,
        context: &PorplContext,
        auth: Option<&str>,
    ) -> Result<Self::Response, PorplError> {
        let u = require_user(context.pool(), context.master_key(), auth).await?;

        Ok(u)
    }
}

#[test]
fn test_validate_username() {
    assert!(validate_username("   a silly little username ").is_err());
    assert!(validate_username("!2~`23132`Acs*9").is_err());
    assert!(validate_username("perfectlyValidUser").is_ok());
}
