use crate::{
    activities::{generate_activity_id, send_tinyboards_activity, verify_person},
    fetcher::user_or_board::UserOrBoard,
    insert_activity,
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::activities::subscribed::{subscribe::Subscribe, undo_subscribe::UndoSubscribe},
  };
  use tinyboards_federation::{
    config::Data,
    kinds::activity::UndoType,
    protocol::verification::verify_urls_match,
    traits::{ActivityHandler, Actor},
  };
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_db::{
    models::{
        board::board_subscriber::{BoardSubscriber, BoardSubscriberForm},
        person::person_subscriber::{PersonSubscriber, PersonSubscriberForm},
    },
    traits::Subscribeable,
  };
  use tinyboards_utils::error::TinyBoardsError;
  use url::Url;
  
  impl UndoSubscribe {
    #[tracing::instrument(skip_all)]
    pub async fn send(
      actor: &ApubPerson,
      board: &ApubBoard,
      context: &Data<TinyBoardsContext>,
    ) -> Result<(), TinyBoardsError> {
      let object = Subscribe::new(actor, board, context)?;
      let undo = UndoSubscribe {
        actor: actor.id().into(),
        to: Some([board.id().into()]),
        object,
        kind: UndoType::Undo,
        id: generate_activity_id(
          UndoType::Undo,
          &context.settings().get_protocol_and_hostname(),
        )?,
      };
      let inbox = vec![board.shared_inbox_or_inbox()];
      send_tinyboards_activity(context, undo, actor, inbox, true).await
    }
  }

  #[async_trait::async_trait]
impl ActivityHandler for UndoSubscribe {
  type DataType = TinyBoardsContext;
  type Error = TinyBoardsError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(&self, context: &Data<TinyBoardsContext>) -> Result<(), TinyBoardsError> {
    verify_urls_match(self.actor.inner(), self.object.actor.inner())?;
    verify_person(&self.actor, context).await?;
    self.object.verify(context).await?;
    if let Some(to) = &self.to {
      verify_urls_match(to[0].inner(), self.object.object.inner())?;
    }
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<TinyBoardsContext>) -> Result<(), TinyBoardsError> {
    insert_activity(&self.id, &self, false, true, context).await?;
    let person = self.actor.dereference(context).await?;
    let object = self.object.object.dereference(context).await?;

    match object {
      UserOrBoard::User(u) => {
        let form = PersonSubscriberForm {
          person_id: u.id,
          subscriber_id: person.id,
          pending: false,
        };
        PersonSubscriber::unsubscribe(context.pool(), &form).await?;
      }
      UserOrBoard::Board(b) => {
        let form = BoardSubscriberForm {
          board_id: b.id,
          person_id: person.id,
          pending: Some(false),
        };
        BoardSubscriber::unsubscribe(context.pool(), &form).await?;
      }
    }

    Ok(())
  }
}