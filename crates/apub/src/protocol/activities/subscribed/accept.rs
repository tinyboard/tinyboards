use crate::{
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::activities::subscribed::subscribe::Subscribe,
  };
  use tinyboards_federation::{
    fetch::object_id::ObjectId,
    kinds::activity::AcceptType,
    protocol::helpers::deserialize_skip_error,
  };
  use serde::{Deserialize, Serialize};
  use url::Url;
  
  #[derive(Clone, Debug, Deserialize, Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct AcceptFollow {
    pub(crate) actor: ObjectId<ApubBoard>,
    /// Optional, for compatibility with platforms that always expect recipient field
    #[serde(deserialize_with = "deserialize_skip_error", default)]
    pub(crate) to: Option<[ObjectId<ApubPerson>; 1]>,
    pub(crate) object: Follow,
    #[serde(rename = "type")]
    pub(crate) kind: AcceptType,
    pub(crate) id: Url,
  }