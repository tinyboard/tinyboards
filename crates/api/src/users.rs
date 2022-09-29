use crate::Perform;
use serde::{Deserialize, Serialize};

use crate::data::PorplContext;
use porpl_db::models::user::User;
use porpl_utils::PorplError;
use regex::Regex;

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

    async fn perform(self, context: &PorplContext) -> Result<Self::Response, PorplError> {
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
    pub message: String,
}

fn validate_username(username: &str) -> bool {
    let re = Regex::new(r"^[A-Za-z][A-Za-z0-9_]{7,29}$").unwrap();
    let valid = re.is_match(username);
    valid
}

#[async_trait::async_trait]
impl Perform for CreateUser {
    type Response = CreateUserResponse;

    async fn perform(self, context: &PorplContext) -> Result<Self::Response, PorplError> {
        let data: CreateUser = self;

        let valid_username = validate_username(&data.username);
        
        if valid_username {

            let _new_user = blocking(context.pool(), move |conn| {
                User::insert(conn, data.username, data.password, data.email)
            })
            .await??;
    
            Ok(CreateUserResponse {
                message: String::from("User created!"),
            })
        } 
        else {
            Err(PorplError::new(409, String::from("Error: Invalid Username")))
        }
    }
}