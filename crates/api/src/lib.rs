pub mod data;
pub mod users;
pub mod utils;
pub mod submissions;
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


#[async_trait::async_trait]
pub trait PerformCrudRead {
    type Response: Serialize;
    type IdType;
    type Form;

    async fn perform_read(
        self,
        context: &PorplContext,
        authorization: Option<&str>,
        id: Self::IdType,
    ) -> Result<Self::Response, PorplError>;

}

#[async_trait::async_trait]
pub trait PerformCrudDelete {
    type IdType;

    async fn perform_delete(
        self,
        context: &PorplContext,
        authorization: Option<&str>,
        id: Self::IdType,
    ) -> Result<usize, PorplError>;
}

#[async_trait::async_trait]
pub trait PerformCrudUpdate {
    type Response: Serialize;
    type IdType;
    type Form;

    async fn perform_update(
        self,
        context: &PorplContext,
        authorization: Option<&str>,
        id: Self::IdType,
        form: &Self::Form,
    ) -> Result<Self::Response, PorplError>;
}

#[async_trait::async_trait]
pub trait PerformCrudCreate {
    type Response: Serialize;
    type IdType;
    type Form;

    async fn perform_create(
        self,
        context: &PorplContext,
        authorization: Option<&str>,
        id: Self::IdType,
        form: &Self::Form,
    ) -> Result<Self::Response, PorplError>;
}
