use actix_web::web::Data;
use porpl_api_common::data::PorplContext;
use porpl_utils::error::PorplError;
use serde::Deserialize;

pub mod user;
pub mod post;

#[async_trait::async_trait(?Send)]
pub trait PerformCrud<'des> {
    type Response: serde::ser::Serialize + Send;
    type Route: Deserialize<'des>;

    async fn perform(
        self,
        context: &Data<PorplContext>,
        authorization: Option<&str>,
    ) -> Result<Self::Response, PorplError>;
}
