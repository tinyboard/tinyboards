use crate::{
    activities::verify_board_matches,
    fetcher::user_or_board::{PersonOrGroupType, UserOrBoard},
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
#[serde(untagged)]
pub(crate) enum AttributedTo {
  TinyBoards(ObjectId<ApubPerson>),
  //Peertube([AttributedToPeertube; 2]),
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

impl Attachment {
  pub(crate) fn new(url: DbUrl) -> Attachment {
    Attachment::Link(Link {
      href: url.into(),
      r#type: Default::default(),
    })
  }

  pub(crate) fn url(self) -> Url {
    match self {
      // url as sent by Tinyboards (new)
      Attachment::Link(l) => l.href,
      // image sent by lotide
      Attachment::Image(i) => i.url,
      // sent by mobilizon
      Attachment::Document(d) => d.url,
    }
  }
}

/// Only allows deserialization if the field is missing or null. If it is present, throws an error.
pub fn deserialize_not_present<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  let result: Option<String> = Deserialize::deserialize(deserializer)?;
  match result {
    None => Ok(None),
    Some(_) => Err(D::Error::custom("Post must not have inReplyTo property")),
  }
}

impl Page {
  /// Only mods can change the post's locked status. So if it is changed from the default value,
  /// it is a mod action and needs to be verified as such.
  ///
  /// Locked needs to be false on a newly created post (verified in [[CreatePost]].
  pub(crate) async fn is_mod_action(
    &self,
    context: &Data<TinyBoardsContext>,
  ) -> Result<bool, TinyBoardsError> {
    let old_post = self.id.clone().dereference_local(context).await;
    Ok(Page::is_locked_changed(&old_post, &self.comments_enabled))
  }

  pub(crate) fn is_locked_changed<E>(
    old_post: &Result<ApubPost, E>,
    new_comments_enabled: &Option<bool>,
  ) -> bool {
    if let Some(new_comments_enabled) = new_comments_enabled {
      if let Ok(old_post) = old_post {
        return new_comments_enabled != &!old_post.is_locked;
      }
    }

    false
  }

  pub(crate) fn creator(&self) -> Result<ObjectId<ApubPerson>, TinyBoardsError> {
    match &self.attributed_to {
      AttributedTo::TinyBoards(l) => Ok(l.clone()),
      // AttributedTo::Peertube(p) => p
      //   .iter()
      //   .find(|a| a.kind == PersonOrGroupType::Person)
      //   .map(|a| ObjectId::<ApubPerson>::from(a.id.clone().into_inner()))
      //   .ok_or_else(|| TinyBoardsError::from_message(400, "page does not specify creator person")),
    }
  }
}


#[async_trait::async_trait]
impl InBoard for Page {
  async fn board(&self, context: &Data<TinyBoardsContext>) -> Result<ApubBoard, TinyBoardsError> {
    let board: ApubBoard = match &self.attributed_to {
      AttributedTo::TinyBoards(_) => {
        let mut iter = self.to.iter().merge(self.cc.iter());
        loop {
          if let Some(bid) = iter.next() {
            let bid = ObjectId::from(bid.clone());
            if let Ok(b) = bid.dereference(context).await {
              break b;
            }
          } else {
            return Err(TinyBoardsError::from_message(400, "no board found in cc"));
          }
        }
      },
      // AttributedTo::Peertube(p) => {
      //   p.iter()
      //     .find(|a| a.kind == PersonOrGroupType::Group)
      //     .map(|a| ObjectId::<ApubBoard>::from(a.id.clone().into_inner()))
      //     .ok_or_else(|| TinyBoardsError::from_message(400, "page does not specify group"))?
      //     .dereference(context)
      //     .await?
      // }
    };
    if let Some(audience) = &self.audience {
      verify_board_matches(audience, board.actor_id.clone())?;
    }
    Ok(board)
  }
}