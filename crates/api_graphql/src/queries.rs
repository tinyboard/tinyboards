use async_graphql::*;
use tinyboards_utils::TinyBoardsError;

use crate::LoggedInUser;

pub struct Query;

#[Object]
impl Query {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    /// Returns the name of the logged in user
    async fn me<'ctx>(&self, ctx: &Context<'ctx>) -> Result<&'ctx str> {
        let logged_in_user = ctx.data::<LoggedInUser>()?;

        match logged_in_user.inner() {
            Some(v) => Ok(v.person.name.as_str()),
            None => Err(TinyBoardsError::from_message(401, "You are not logged in.").into()),
        }
    }
}
