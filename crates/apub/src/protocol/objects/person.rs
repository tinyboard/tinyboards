use crate::{
    objects::person::ApubPerson,
    protocol::{objects::Endpoints, ImageObject, Source},
};
use tinyboards_federation::{
    fetch::object_id::ObjectId,
    protocol::{helpers::deserialize_skip_error, public_key::PublicKey},
};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum UserTypes {
  Person,
  Service,
  Organization,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(rename = "type")]
    pub(crate) kind: UserTypes,
    pub(crate) id: ObjectId<ApubPerson>,
    /// username, set at account creation
    pub(crate) preferred_username: String,
    pub(crate) inbox: Url,
    pub(crate) outbox: Url,
    pub(crate) public_key: PublicKey,
    /// display_name
    pub(crate) name: Option<String>,
    pub(crate) summary: Option<String>,
    #[serde(deserialize_with = "deserialize_skip_error", default)]
    pub(crate) source: Option<Source>,
    /// user avatar
    pub(crate) icon: Option<ImageObject>,
    /// user banner
    pub(crate) image: Option<ImageObject>,
    pub(crate) matrix_user_id: Option<String>,
    pub(crate) endpoints: Option<Endpoints>,
    pub(crate) published: Option<DateTime<FixedOffset>>,
    pub(crate) updated: Option<DateTime<FixedOffset>>,
}