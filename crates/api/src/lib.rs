pub mod data;
pub mod error;
pub mod post;
pub mod utils;
use error::PorplError;

use data::PorplContext;
use serde::Serialize;

#[async_trait::async_trait]
pub trait Perform {
    type Response: Serialize;

    async fn perform(&self, context: &PorplContext) -> Result<Self::Response, PorplError>;
}
