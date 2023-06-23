pub mod comment;
pub mod comment_report;
pub mod moderator;
pub mod admin;
pub mod post;
pub mod post_report;
pub mod site;
pub mod local_user;
pub mod board;
use actix_web::{web::Data, HttpResponse, HttpRequest};
use tinyboards_utils::TinyBoardsError;

use serde::{Deserialize, Serialize};
use tinyboards_api_common::data::TinyBoardsContext;

#[async_trait::async_trait(?Send)]
pub trait Perform<'des> {
    type Response: Serialize;
    type Route: Deserialize<'des>;

    /**
     *   Fn that performs the operation. Takes a `TinyBoardsContext` object (for the db connection) and an `Option<&str>` which might contain the `Authorization` header. If you're implementing this on an operation that doesn't require (or would benefit from) a logged in user, you don't need the last argument, therefore you can ignore it with the `_` pattern:
     *   ```
     *   async fn perform(self, context: &TinyBoardsContext, _: Option<&str>) -> Result<Self::Response, TinyBoardsError> { ... }
     *   ```
     */
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError>;
}

#[async_trait::async_trait(?Send)]
pub trait PerformUpload<'des> {
    type Response: Serialize;
    type Route: Deserialize<'des>;

    async fn perform_upload(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError>;
}

pub(crate) fn check_report_reason(reason: &str) -> Result<(), TinyBoardsError> {
    if reason.is_empty() {
        return Err(TinyBoardsError::from_message(400, "report reason required."));
    }
    if reason.chars().count() > 1000 {
        return Err(TinyBoardsError::from_message(400, "report reason is too long."))
    }
    Ok(())
}