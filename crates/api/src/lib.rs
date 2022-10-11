pub mod local_user;
use actix_web::web::Data;
use porpl_utils::PorplError;

use porpl_api_common::data::PorplContext;
use serde::{Deserialize, Serialize};

#[async_trait::async_trait(?Send)]
pub trait Perform<'des> {
    type Response: Serialize;
    type Route: Deserialize<'des>;

    /**
     *   Fn that performs the operation. Takes a `PorplContext` object (for the db connection) and an `Option<&str>` which might contain the `Authorization` header. If you're implementing this on an operation that doesn't require (or would benefit from) a logged in user, you don't need the last argument, therefore you can ignore it with the `_` pattern:
     *   ```
     *   async fn perform(self, context: &PorplContext, _: Option<&str>) -> Result<Self::Response, PorplError> { ... }
     *   ```
     */
    async fn perform(
        self,
        context: &Data<PorplContext>,
        path: Self::Route,
        authorization: Option<&str>,
    ) -> Result<Self::Response, PorplError>;
}
