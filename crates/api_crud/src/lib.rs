use actix_web::web::Data;
use porpl_api_common::data::PorplContext;
use porpl_utils::error::PorplError;
use serde::Deserialize;

pub mod comment;
pub mod post;
pub mod user;

#[async_trait::async_trait(?Send)]
pub trait PerformCrud<'des> {
    type Response: serde::ser::Serialize + Send;
    type Route: Deserialize<'des>;

    async fn perform(
        self,
        context: &Data<PorplContext>,
        path: Self::Route,
        authorization: Option<&str>,
    ) -> Result<Self::Response, PorplError>;
}
