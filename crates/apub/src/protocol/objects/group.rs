use crate::{
    check_ap_id_valid_with_strictness,
    collections::{
      board_moderators::ApubBoardModerators,
      board_outbox::ApubBoardOutbox,
      board_featured::ApubBoardFeatured,
    },
    fetch_local_site_data,
    objects::{board::ApubBoard, read_from_string_or_source_opt},
    protocol::{
      objects::{Endpoints, LanguageTag},
      ImageObject,
      Source,
    },
  };
  use tinyboards_federation::{
    fetch::{collection_id::CollectionId, object_id::ObjectId},
    kinds::actor::GroupType,
    protocol::{
      helpers::deserialize_skip_error,
      public_key::PublicKey,
      verification::verify_domains_match,
    },
  };
  use chrono::{DateTime, FixedOffset};
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_db::{
    models::board::boards::BoardForm,
    utils::naive_now,
  };
  use tinyboards_utils::error::TinyBoardsError;
  use serde::{Deserialize, Serialize};
  use serde_with::skip_serializing_none;
  use std::fmt::Debug;
  use url::Url;

  #[skip_serializing_none]
  #[derive(Clone, Debug, Deserialize, Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Group {
    #[serde(rename = "type")]
    pub(crate) kind: GroupType,
    pub(crate) id: ObjectId<ApubBoard>,
    /// username
    pub(crate) preferred_username: String,
    pub(crate) inbox: Url,
    pub(crate) followers: Url,
    pub(crate) public_key: PublicKey,
    /// title
    pub(crate) name: Option<String>,
    pub(crate) summary: Option<String>,
    #[serde(deserialize_with = "deserialize_skip_error", default)]
    pub(crate) source: Option<Source>,
    pub(crate) icon: Option<ImageObject>,
    /// banner
    pub(crate) image: Option<ImageObject>,
    pub(crate) sensitive: Option<bool>,
    #[serde(deserialize_with = "deserialize_skip_error", default)]
    pub(crate) attributed_to: Option<CollectionId<ApubBoardModerators>>,
    // tb extension
    pub(crate) posting_restricted_to_mods: Option<bool>,
    pub(crate) outbox: CollectionId<ApubBoardOutbox>,
    pub(crate) endpoints: Option<Endpoints>,
    pub(crate) featured: Option<CollectionId<ApubBoardFeatured>>,
    #[serde(default)]
    pub(crate) language: Vec<LanguageTag>,
    pub(crate) published: Option<DateTime<FixedOffset>>,
    pub(crate) updated: Option<DateTime<FixedOffset>>,
  }

  impl Group {
    pub(crate) async fn verify(
        &self,
        expected_domain: &Url,
        context: &TinyBoardsContext
    ) -> Result<(), TinyBoardsError> {
        let local_site_data = fetch_local_site_data(context.pool()).await?;

        check_ap_id_valid_with_strictness(
            self.id.inner(), 
            true, 
            &local_site_data, 
            context.settings()
        )?;
        
        verify_domains_match(expected_domain, self.id.inner())?;

        Ok(())
    }

    pub(crate) fn into_form(self, instance_id: i32) -> BoardForm {
        BoardForm {
            name: Some(self.preferred_username.clone()),
            title: Some(self.name.unwrap_or(self.preferred_username)),
            description: Some(read_from_string_or_source_opt(&self.summary, &None, &self.source)),
            is_deleted: None,
            is_removed: None,
            updated: Some(self.updated.map(|u| u.naive_local())),
            is_hidden: None,
            is_banned: None,
            private_key: None,
            public_key: Some(self.public_key.public_key_pem),
            last_refreshed_date: Some(naive_now()),
            is_nsfw: None,
            subscribers_url: Some(self.followers.into()),
            inbox_url: Some(self.inbox.into()),
            shared_inbox_url: Some(self.endpoints.map(|e| e.shared_inbox.into())),
            moderators_url: self.attributed_to.map(Into::into),
            posting_restricted_to_mods: self.posting_restricted_to_mods,
            instance_id: Some(instance_id),
            featured_url: self.featured.map(Into::into),
            local: Some(false),
            actor_id: Some(self.id.into()),
            icon: self.icon.map(|i| i.url.into()),
            banner: self.image.map(|i| i.url.into()),
        }
    }
  }