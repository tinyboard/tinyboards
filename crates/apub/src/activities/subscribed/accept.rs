use crate::{
    activities::{generate_activity_id, send_tinyboards_activity},
    insert_activity,
    protocol::activities::subscribed::{accept::AcceptSubscribe, subscribe::Subscribe},
  };
  use tinyboards_federation::{
    config::Data,
    kinds::activity::AcceptType,
    protocol::verification::verify_urls_match,
    traits::{ActivityHandler, Actor},
  };
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_db::{models::board::board_subscriber::BoardSubscriber, traits::Subscribeable};
  use tinyboards_utils::error::TinyBoardsError;
  use url::Url;

  impl AcceptSubscribe {
    #[tracing::instrument(skip_all)]
    pub async fn send(subscribe: Subscribe, context: &Data<TinyBoardsContext>) -> Result<(), TinyBoardsError> {
      let user_or_board = subscribe.object.dereference_local(context).await?;
      let person = subscribe.actor.clone().dereference(context).await?;
      let accept = AcceptSubscribe {
        actor: user_or_board.id().into(),
        to: Some([person.id().into()]),
        object: subscribe,
        kind: AcceptType::Accept,
        id: generate_activity_id(
          AcceptType::Accept,
          &context.settings().get_protocol_and_hostname(),
        )?,
      };
      let inbox = vec![person.shared_inbox_or_inbox()];
      send_tinyboards_activity(context, accept, &user_or_board, inbox, true).await
    }
  }
  
/// Handle accepted subscribes
#[async_trait::async_trait]
impl ActivityHandler for AcceptSubscribe {
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
    verify_urls_match(self.actor.inner(), self.object.object.inner())?;
    self.object.verify(context).await?;
    if let Some(to) = &self.to {
      verify_urls_match(to[0].inner(), self.object.actor.inner())?;
    }
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<TinyBoardsContext>) -> Result<(), TinyBoardsError> {
    insert_activity(&self.id, &self, false, true, context).await?;
    let board = self.actor.dereference(context).await?;
    let person = self.object.actor.dereference(context).await?;
    // This will throw an error if no subscribe was requested
    let board_id = board.id;
    let person_id = person.id;
    BoardSubscriber::subscribe_accepted(context.pool(), board_id, person_id).await?;

    Ok(())
  }
}