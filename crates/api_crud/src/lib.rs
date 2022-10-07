use actix_web::{web, web::Data};
use porpl_api_common::{person::*};
use porpl_utils::{error::PorplError};
use serde::Deserialize;
use porpl_api::data::PorplContext;

mod user;


#[async_trait::async_trait(?Send)]
pub trait PerformCrud {
    type Response: serde::ser::Serialize + Send;

    async fn perform(
        &self,
        context: &Data<PorplContext>
    ) -> Result<Self::Response, PorplError>;
}