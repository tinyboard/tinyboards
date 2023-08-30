use actix_web::web::Data;
use serde::Deserialize;
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_utils::error::TinyBoardsError;

pub mod applications;
pub mod board;
pub mod comment;
pub mod comment_report;
pub mod emoji;
pub mod invite;
pub mod message;
pub mod post;
pub mod post_report;
pub mod user;

#[async_trait::async_trait(?Send)]
pub trait PerformCrud<'des> {
    type Response: serde::ser::Serialize + Send;
    type Route: Deserialize<'des>;

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        authorization: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError>;
}
