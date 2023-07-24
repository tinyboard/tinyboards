use actix_web::web::Data;
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_utils::error::TinyBoardsError;
use serde::Deserialize;

pub mod board;
pub mod comment;
pub mod emoji;
pub mod post;
pub mod user;
pub mod invite;
pub mod applications;
pub mod post_report;
pub mod comment_report;

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
