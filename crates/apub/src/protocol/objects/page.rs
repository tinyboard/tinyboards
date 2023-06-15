use crate::{
    activities::verify_board_matches,
    fetcher::user_or_community::{PersonOrGroupType, UserOrCommunity},
    objects::{board::ApubBoard, person::ApubPerson, post::ApubPost},
    protocol::{objects::LanguageTag, ImageObject, InBoard, Source},
  };
  use tinyboards_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{
      link::LinkType,
      object::{DocumentType, ImageType},
    },
    protocol::{
      helpers::{deserialize_one_or_many, deserialize_skip_error},
      values::MediaTypeMarkdownOrHtml,
    },
    traits::{ActivityHandler, Object},
  };
  use chrono::{DateTime, FixedOffset};
  use itertools::Itertools;
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_db::newtypes::DbUrl;
  use tinyboards_utils::error::TinyBoardsError;
  use serde::{de::Error, Deserialize, Deserializer, Serialize};
  use serde_with::skip_serializing_none;
  use url::Url;



#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum PageType {
  Page,
  Article,
  Note,
  Video,
  Event,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
  #[serde(rename = "type")]
  pub(crate) kind: PageType,
  pub(crate) id: ObjectId<ApubPost>,
  pub(crate) attributed_to: AttributedTo,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  // If there is inReplyTo field this is actually a comment and must not be parsed
  #[serde(deserialize_with = "deserialize_not_present", default)]
  pub(crate) in_reply_to: Option<String>,
  pub(crate) name: Option<String>,
  #[serde(deserialize_with = "deserialize_one_or_many", default)]
  pub(crate) cc: Vec<Url>,
  pub(crate) content: Option<String>,
  pub(crate) media_type: Option<MediaTypeMarkdownOrHtml>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) source: Option<Source>,
  /// most apub software uses array for attachments, so we can do the same (only use the first element)
  #[serde(default)]
  pub(crate) attachment: Vec<Attachment>,
  pub(crate) image: Option<ImageObject>,
  pub(crate) comments_enabled: Option<bool>,
  pub(crate) sensitive: Option<bool>,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
  pub(crate) language: Option<LanguageTag>,
  pub(crate) audience: Option<ObjectId<ApubBoard>>,
}











#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Link {
  pub(crate) href: Url,
  pub(crate) r#type: LinkType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Image {
  #[serde(rename = "type")]
  pub(crate) kind: ImageType,
  pub(crate) url: Url,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Document {
  #[serde(rename = "type")]
  pub(crate) kind: DocumentType,
  pub(crate) url: Url,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum Attachment {
  Link(Link),
  Image(Image),
  Document(Document),
}