use crate::{objects::board::ApubBoard, protocol::{IdOrNestedObject}};
use tinyboards_federation::{
    fetch::object_id::ObjectId,
    kinds::activity::AnnounceType,
    protocol::helpers::deserialize_one_or_many,
};
use serde::{Serialize, Deserialize};
use serde_json::{Map, Value};
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnounceActivity {
  pub(crate) actor: ObjectId<ApubBoard>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: IdOrNestedObject<RawAnnouncableActivities>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: AnnounceType,
  pub(crate) id: Url,
}

/// Use this to receive board inbox activities, and then announce them if valid. This
/// ensures that all json fields are kept, even if Tinyboards doesnt understand them.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawAnnouncableActivities {
  pub(crate) id: Url,
  pub(crate) actor: Url,
  #[serde(flatten)]
  pub(crate) other: Map<String, Value>,
}

