use crate::{
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::objects::{group::Group, person::Person},
  };
  use tinyboards_federation::{
    config::Data,
    traits::{Actor, Object},
  };
  use chrono::NaiveDateTime;
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_utils::error::TinyBoardsError;
  use serde::{Deserialize, Serialize};
  use url::Url;
  
  #[derive(Clone, Debug)]
  pub enum UserOrBoard {
    User(ApubPerson),
    Board(ApubBoard),
  }
  
  #[derive(Serialize, Deserialize, Clone, Debug)]
  #[serde(untagged)]
  pub enum PersonOrGroup {
    Person(Person),
    Group(Group),
  }
  
  #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
  pub enum PersonOrGroupType {
    Person,
    Group,
  }
  
  #[async_trait::async_trait]
  impl Object for UserOrBoard {
    type DataType = TinyBoardsContext;
    type Kind = PersonOrGroup;
    type Error = TinyBoardsError;
  
    fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
      Some(match self {
        UserOrBoard::User(p) => p.last_refreshed_date,
        UserOrBoard::Board(p) => p.last_refreshed_date,
      })
    }
  
    #[tracing::instrument(skip_all)]
    async fn read_from_id(
      object_id: Url,
      data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, TinyBoardsError> {
      let person = ApubPerson::read_from_id(object_id.clone(), data).await?;
      Ok(match person {
        Some(o) => Some(UserOrBoard::User(o)),
        None => ApubBoard::read_from_id(object_id, data)
          .await?
          .map(UserOrBoard::Board),
      })
    }
  
    #[tracing::instrument(skip_all)]
    async fn delete(self, data: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
      match self {
        UserOrBoard::User(p) => p.delete(data).await,
        UserOrBoard::Board(p) => p.delete(data).await,
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
        PersonOrGroup::Person(a) => ApubPerson::verify(a, expected_domain, data).await,
        PersonOrGroup::Group(a) => ApubBoard::verify(a, expected_domain, data).await,
      }
    }
  
    #[tracing::instrument(skip_all)]
    async fn from_json(apub: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, TinyBoardsError> {
      Ok(match apub {
        PersonOrGroup::Person(p) => UserOrBoard::User(ApubPerson::from_json(p, data).await?),
        PersonOrGroup::Group(p) => {
          UserOrBoard::Board(ApubBoard::from_json(p, data).await?)
        }
      })
    }
  }
  
  impl Actor for UserOrBoard {
    fn id(&self) -> Url {
      match self {
        UserOrBoard::User(u) => u.id(),
        UserOrBoard::Board(c) => c.id(),
      }
    }
  
    fn public_key_pem(&self) -> &str {
      match self {
        UserOrBoard::User(p) => p.public_key_pem(),
        UserOrBoard::Board(p) => p.public_key_pem(),
      }
    }
  
    fn private_key_pem(&self) -> Option<String> {
      match self {
        UserOrBoard::User(p) => p.private_key_pem(),
        UserOrBoard::Board(p) => p.private_key_pem(),
      }
    }
  
    fn inbox(&self) -> Url {
      unimplemented!()
    }
  }