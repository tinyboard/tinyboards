pub mod data;
pub mod users;
pub mod utils;
use porpl_utils::PorplError;

use data::PorplContext;
use serde::Serialize;

#[async_trait::async_trait]
pub trait Perform {
    type Response: Serialize;

    async fn perform(self, context: &PorplContext) -> Result<Self::Response, PorplError>;
}

// #[async_trait::async_trait]
// pub trait PerformInsert {
//     type Response: Serialize;

//     async fn perform_insert(&self, context: &PorplContext, user_form: &porpl_db::InsertUser) -> Result<Self::Response, PorplError>;
// }
