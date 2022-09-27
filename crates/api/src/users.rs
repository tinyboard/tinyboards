use crate::{Perform, PorplError};
//use crate::PerformInsert;
use serde::{Deserialize, Serialize};

use crate::data::PorplContext;
use porpl_db::models::users::Users;
//use porpl_db::models::users::InsertUser;

use crate::utils::blocking;

#[derive(Deserialize)]
pub struct GetUsers {
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct GetUsersResponse {
    listing: Vec<Users>,
}

#[derive(Serialize)]
pub struct GetInsertUserToDbResponse {
    rows_returned: Option<usize>,
}

#[async_trait::async_trait]
impl Perform for GetUsers {
    type Response = GetUsersResponse;

    async fn perform(&self, context: &PorplContext) -> Result<Self::Response, PorplError> {

        let data: &GetUsers = self;

        let limit: i64 = match data.limit {
            Some(n) => n,
            None => 25
        };

        let users = blocking(&context.pool(), move|conn| Users::load(conn, limit)).await?;

        Ok(GetUsersResponse { listing: users })
    }
}


// has an error, but I think this theoretically should be the right way to implement this function... (still learning)

// #[async_trait::async_trait]
// impl PerformInsert for InsertUser {
//     type Response = GetInsertUserToDbResponse;

//     async fn perform_insert(&self, context: &PorplContext, user_form: &InsertUser) -> Result<Self::Response, PorplError> {

//         let result = blocking(&context.pool(), move|conn| InsertUser::insert(conn, &user_form)).await?;

//         Ok(GetInsertUserToDbResponse { rows_returned: result.ok() })

//     }
// }