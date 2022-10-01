pub mod data;
pub mod users;
pub mod utils;
pub mod submit;
use porpl_utils::PorplError;

use data::PorplContext;
use serde::Serialize;

#[async_trait::async_trait]
pub trait Perform {
    type Response: Serialize;
   
    /**
     *   Fn that performs the operation. Takes a `PorplContext` object (for the db connection) and an `Option<&str>` which might contain the `Authorization` header. If you're implementing this on an operation that doesn't require (or would benefit from) a logged in user, you don't need the last argument, therefore you can ignore it with the `_` pattern:
     *   ```
     *   async fn perform(self, context: &PorplContext, _: Option<&str>) -> Result<Self::Response, PorplError> { ... }
     *   ```
    */
    async fn perform(
        self,
        context: &PorplContext,
        authorization: Option<&str>,
    ) -> Result<Self::Response, PorplError>;
}
