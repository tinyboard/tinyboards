pub mod user;
pub mod post;
pub mod comment;
use actix_web::web::Data;
use tinyboards_utils::TinyBoardsError;

use tinyboards_api_common::data::TinyBoardsContext;
use serde::{Deserialize, Serialize};

#[async_trait::async_trait(?Send)]
pub trait Perform<'des> {
    type Response: Serialize;
    type Route: Deserialize<'des>;

    /**
     *   Fn that performs the operation. Takes a `TinyBoardsContext` object (for the db connection) and an `Option<&str>` which might contain the `Authorization` header. If you're implementing this on an operation that doesn't require (or would benefit from) a logged in user, you don't need the last argument, therefore you can ignore it with the `_` pattern:
     *   ```
     *   async fn perform(self, context: &TinyBoardsContext, _: Option<&str>) -> Result<Self::Response, TinyBoardsError> { ... }
     *   ```
     */
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        authorization: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError>;
}
