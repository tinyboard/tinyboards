use crate::{
    objects::{comment::ApubComment, board::ApubBoard, person::ApubPerson, post::ApubPost},
    protocol::objects::{group::Group, note::Note, page::Page, person::Person},
  };
  use tinyboards_federation::{
    config::Data,
    fetch::{object_id::ObjectId, webfinger::webfinger_resolve_actor},
    traits::Object,
  };
  use chrono::NaiveDateTime;
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_utils::error::TinyBoardsError;
  use serde::Deserialize;
  use url::Url;
  
  /// Converts search query to object id. The query can either be an URL, which will be treated as
  /// ObjectId directly, or a webfinger identifier (@user@example.com or !community@example.com)
  /// which gets resolved to an URL.
  #[tracing::instrument(skip_all)]
  pub(crate) async fn search_query_to_object_id(
    query: &str,
    context: &Data<TinyBoardsContext>,
  ) -> Result<SearchableObjects, TinyBoardsError> {
    Ok(match Url::parse(query) {
      Ok(url) => {
        // its already an url, just go with it
        ObjectId::from(url).dereference(context).await?
      }
      Err(_) => {
        // not an url, try to resolve via webfinger
        let mut chars = query.chars();
        let kind = chars.next();
        let identifier = chars.as_str();
        match kind {
          Some('@') => SearchableObjects::Person(
            webfinger_resolve_actor::<TinyBoardsContext, ApubPerson>(identifier, context).await?,
          ),
          Some('!') => SearchableObjects::Community(
            webfinger_resolve_actor::<TinyBoardsContext, ApubBoard>(identifier, context).await?,
          ),
          _ => return Err(TinyBoardsError::from_message("invalid query")),
        }
      }
    })
  }
  
  /// The types of ActivityPub objects that can be fetched directly by searching for their ID.
  #[derive(Debug)]
  pub(crate) enum SearchableObjects {
    Person(ApubPerson),
    Board(ApubBoard),
    Post(ApubPost),
    Comment(ApubComment),
  }
  
  #[derive(Deserialize)]
  #[serde(untagged)]
  pub(crate) enum SearchableKinds {
    Group(Group),
    Person(Person),
    Page(Page),
    Note(Note),
  }
  
  #[async_trait::async_trait]
  impl Object for SearchableObjects {
    type DataType = TinyBoardsContext;
    type Kind = SearchableKinds;
    type Error = TinyBoardsError;
  
    fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
      match self {
        SearchableObjects::Person(p) => p.last_refreshed_at(),
        SearchableObjects::Board(c) => c.last_refreshed_at(),
        SearchableObjects::Post(p) => p.last_refreshed_at(),
        SearchableObjects::Comment(c) => c.last_refreshed_at(),
      }
    }
  
    // TODO: figure out how to check all 4 objects in one query
    #[tracing::instrument(skip_all)]
    async fn read_from_id(
      object_id: Url,
      context: &Data<Self::DataType>,
    ) -> Result<Option<Self>, TinyBoardsError> {
      let b = ApubBoard::read_from_id(object_id.clone(), context).await?;
      if let Some(b) = b {
        return Ok(Some(SearchableObjects::Board(b)));
      }
      let p = ApubPerson::read_from_id(object_id.clone(), context).await?;
      if let Some(p) = p {
        return Ok(Some(SearchableObjects::Person(p)));
      }
      let p = ApubPost::read_from_id(object_id.clone(), context).await?;
      if let Some(p) = p {
        return Ok(Some(SearchableObjects::Post(p)));
      }
      let c = ApubComment::read_from_id(object_id, context).await?;
      if let Some(c) = c {
        return Ok(Some(SearchableObjects::Comment(c)));
      }
      Ok(None)
    }
  
    #[tracing::instrument(skip_all)]
    async fn delete(self, data: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
      match self {
        SearchableObjects::Person(p) => p.delete(data).await,
        SearchableObjects::Community(c) => c.delete(data).await,
        SearchableObjects::Post(p) => p.delete(data).await,
        SearchableObjects::Comment(c) => c.delete(data).await,
      }
    }
  
    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, TinyBoardsError> {
      unimplemented!()
    }
  
    #[tracing::instrument(skip_all)]
    async fn verify(
      apub: &Self::Kind,
      expected_domain: &Url,
      data: &Data<Self::DataType>,
    ) -> Result<(), TinyBoardsError> {
      match apub {
        SearchableKinds::Group(a) => ApubBoard::verify(a, expected_domain, data).await,
        SearchableKinds::Person(a) => ApubPerson::verify(a, expected_domain, data).await,
        SearchableKinds::Page(a) => ApubPost::verify(a, expected_domain, data).await,
        SearchableKinds::Note(a) => ApubComment::verify(a, expected_domain, data).await,
      }
    }
  
    #[tracing::instrument(skip_all)]
    async fn from_json(apub: Self::Kind, context: &Data<TinyBoardsContext>) -> Result<Self, TinyBoardsError> {
      use SearchableKinds as SAT;
      use SearchableObjects as SO;
      Ok(match apub {
        SAT::Group(g) => SO::Community(ApubBoard::from_json(g, context).await?),
        SAT::Person(p) => SO::Person(ApubPerson::from_json(p, context).await?),
        SAT::Page(p) => SO::Post(ApubPost::from_json(p, context).await?),
        SAT::Note(n) => SO::Comment(ApubComment::from_json(n, context).await?),
      })
    }
  }