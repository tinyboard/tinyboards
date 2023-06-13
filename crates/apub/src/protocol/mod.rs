//use crate::objects::board::ApubBoard;
use tinyboards_federation::{
  config::Data,
  fetch::fetch_object_http,
  kinds::object::ImageType,
  protocol::values::MediaTypeMarkdown,
};
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_db::newtypes::DbUrl;
use tinyboards_utils::error::TinyBoardsError;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;


pub mod activities;
pub mod collections;
pub mod objects;



#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
  pub(crate) content: String,
  pub(crate) media_type: MediaTypeMarkdown,
}

impl Source {
  pub(crate) fn new(content: String) -> Self {
    Source {
      content,
      media_type: MediaTypeMarkdown::Markdown,
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
  #[serde(rename = "type")]
  kind: ImageType,
  pub(crate) url: Url,
}

impl ImageObject {
  pub(crate) fn new(url: DbUrl) -> Self {
    ImageObject {
      kind: ImageType::Image,
      url: url.into(),
    }
  }
}