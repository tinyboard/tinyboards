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

use crate::objects::board::ApubBoard;


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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Unparsed(HashMap<String, serde_json::Value>);

pub(crate) trait Id {
  fn object_id(&self) -> &Url;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum IdOrNestedObject<Kind: Id> {
  Id(Url),
  NestedObject(Kind),
}

impl<Kind: Id + DeserializeOwned + Send> IdOrNestedObject<Kind> {
  pub(crate) fn id(&self) -> &Url {
    match self {
      IdOrNestedObject::Id(i) => i,
      IdOrNestedObject::NestedObject(n) => n.object_id(),
    }
  }
  pub(crate) async fn object(self, context: &Data<TinyBoardsContext>) -> Result<Kind, TinyBoardsError> {
    match self {
      IdOrNestedObject::Id(i) => Ok(fetch_object_http(&i, context).await?),
      IdOrNestedObject::NestedObject(o) => Ok(o),
    }
  }
}

#[async_trait::async_trait]
pub trait InBoard {
  async fn board(&self, context: &Data<TinyBoardsContext>) -> Result<ApubBoard, TinyBoardsError>;
}