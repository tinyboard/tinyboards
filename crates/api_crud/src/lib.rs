use actix_web::web::Data;
use serde::Deserialize;
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_utils::error::TinyBoardsError;

pub mod applications;
pub mod board;
pub mod board_subscriptions;
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
    type Route: Deserialize<'des> + Clone;

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        authorization: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError>;
}

pub(crate) fn check_report_reason(reason: &str) -> Result<(), TinyBoardsError> {
    if reason.is_empty() {
        return Err(TinyBoardsError::from_message(
            400,
            "report reason required.",
        ));
    }
    if reason.chars().count() > 1000 {
        return Err(TinyBoardsError::from_message(
            400,
            "report reason is too long.",
        ));
    }
    Ok(())
}
