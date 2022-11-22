// use actix_web::{
//     body::BodyStream,
//     error,
//     http::{
//         header::{HeaderName, ACCEPT_ENCODING, HOST},
//         StatusCode,
//     },
//     web,
//     Error,
//     HttpRequest,
//     HttpResponse,
// };
// use futures::stream::{Stream, StreamExt};
// use tinyboards_api_common::utils::get_user_view_from_jwt;
// use tinyboards_api_common::data::TinyBoardsContext;
// use tinyboards_db::models::site::site::Site;
// use tinyboards_utils::{claims::Claims, rate_limit};
// use reqwest::Body;
// use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
// use serde::{Deserialize, Serialize};

// pub fn config(
//     cfg: &mut web::ServiceConfig,
//     client: ClientWithMiddleware,
//     rate_limit:
// )