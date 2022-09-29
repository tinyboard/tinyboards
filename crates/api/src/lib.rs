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
