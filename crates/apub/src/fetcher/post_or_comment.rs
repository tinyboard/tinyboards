use crate::{
    objects::{comment::ApubComment, board::ApubBoard, post::ApubPost},
    protocol::{
      objects::{note::Note, page::Page},
      InBoard,
    },
  };
  use tinyboards_federation::{config::Data, traits::Object};
  use chrono::NaiveDateTime;
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_db::{
    models::{board::boards::Board, post::posts::Post},
    traits::Crud,
  };
  use tinyboards_utils::error::TinyBoardsError;
  use serde::Deserialize;
  use url::Url;
  
  #[derive(Clone, Debug)]
  pub enum PostOrComment {
    Post(ApubPost),
    Comment(ApubComment),
  }
  
  #[derive(Deserialize)]
  #[serde(untagged)]
  pub enum PageOrNote {
    Page(Box<Page>),
    Note(Note),
  }
  
  #[async_trait::async_trait]
  impl Object for PostOrComment {
    type DataType = TinyBoardsContext;
    type Kind = PageOrNote;
    type Error = TinyBoardsError;
  
    fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
      None
    }
  
    #[tracing::instrument(skip_all)]
    async fn read_from_id(
      object_id: Url,
      data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, TinyBoardsError> {
      let post = ApubPost::read_from_id(object_id.clone(), data).await?;
      Ok(match post {
        Some(o) => Some(PostOrComment::Post(o)),
        None => ApubComment::read_from_id(object_id, data)
          .await?
          .map(PostOrComment::Comment),
      })
    }
  
    #[tracing::instrument(skip_all)]
    async fn delete(self, data: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
      match self {
        PostOrComment::Post(p) => p.delete(data).await,
        PostOrComment::Comment(c) => c.delete(data).await,
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
        PageOrNote::Page(a) => ApubPost::verify(a, expected_domain, data).await,
        PageOrNote::Note(a) => ApubComment::verify(a, expected_domain, data).await,
      }
    }
  
    #[tracing::instrument(skip_all)]
    async fn from_json(apub: PageOrNote, context: &Data<TinyBoardsContext>) -> Result<Self, TinyBoardsError> {
      Ok(match apub {
        PageOrNote::Page(p) => PostOrComment::Post(ApubPost::from_json(*p, context).await?),
        PageOrNote::Note(n) => PostOrComment::Comment(ApubComment::from_json(n, context).await?),
      })
    }
  }
  
  #[async_trait::async_trait]
  impl InBoard for PostOrComment {
    async fn board(&self, context: &Data<TinyBoardsContext>) -> Result<ApubBoard, TinyBoardsError> {
      let board_id = match self {
        PostOrComment::Post(p) => p.community_id,
        PostOrComment::Comment(c) => Post::read(context.pool(), c.post_id).await?.community_id,
      };
      Ok(Board::read(context.pool(), board_id).await?.into())
    }
  }