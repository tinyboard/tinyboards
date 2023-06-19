use crate::{
    activities::{
        generate_activity_id,
        send_tinyboards_activity,
        verify_is_public,
        verify_person_in_board,
    },
    activity_lists::AnnouncableActivities,
    insert_activity,
    objects::board::ApubBoard,
    protocol::{
        activities::board::announce::{AnnounceActivity, RawAnnouncableActivities},
        Id,
        IdOrNestedObject,
        InBoard
    },
};
use tinyboards_federation::{
    config::Data,
    kinds::{activity::AnnounceType, public},
    traits::{ActivityHandler, Actor},
  };
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_utils::error::TinyBoardsError;
  use serde_json::Value;
  use url::Url;

  #[async_trait::async_trait]
impl ActivityHandler for RawAnnouncableActivities {
  type DataType = TinyBoardsContext;
  type Error = TinyBoardsError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    &self.actor
  }

  #[tracing::instrument(skip_all)]
  async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
    let activity: AnnouncableActivities = self.clone().try_into()?;
    // This is only for sending, not receiving so we reject it.
    if let AnnouncableActivities::Page(_) = activity {
      return Err(TinyBoardsError::from_message(400, "can't receive page"));
    }
    let board = activity.board(data).await?;
    let actor_id = activity.actor().clone().into();

    // verify and receive activity
    activity.verify(data).await?;
    activity.receive(data).await?;

    // send to community followers
    if board.local {
      verify_person_in_board(&actor_id, &board, data).await?;
      AnnounceActivity::send(self, &board, data).await?;
    }
    Ok(())
  }
}

impl AnnounceActivity {
    pub(crate) fn new(
        object: RawAnnouncableActivities,
        board: &ApubBoard,
        context: &Data<TinyBoardsContext>,
    ) -> Result<AnnounceActivity, TinyBoardsError> {
        Ok(
            AnnounceActivity { 
                actor: board.id().into(), 
                to: vec![public()], 
                object: IdOrNestedObject::NestedObject(object), 
                cc: vec![board.subscribers_url.clone().into()], 
                kind: AnnounceType::Announce, 
                id: generate_activity_id(
                    &AnnounceType::Announce, 
                    &context.settings().get_protocol_and_hostname(),
                )?, 
            })
    }

    #[tracing::instrument(skip_all)]
    pub async fn send(
        object: RawAnnouncableActivities,
        board: &ApubBoard,
        context: &Data<TinyBoardsContext>,
    ) -> Result<(), TinyBoardsError> {
        
        let announce = AnnounceActivity::new(object.clone(), board, context)?;
        let inboxes = board.get_subscriber_inboxes(context).await?;
        send_tinyboards_activity(context, announce, board, inboxes.clone(), false).await?;

        Ok(())
    }
}


#[async_trait::async_trait]
impl ActivityHandler for AnnounceActivity {
    type DataType = TinyBoardsContext;
    type Error = TinyBoardsError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    #[tracing::instrument(skip_all)]
    async fn verify(&self, _context: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
        verify_is_public(&self.to, &self.cc)?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
      insert_activity(&self.id, &self, false, false, context).await?;
      let object: AnnouncableActivities = self.object.object(context).await?.try_into()?;
      // This is only for sending, not receiving so we reject it.
      if let AnnouncableActivities::Page(_) = object {
        return Err(TinyBoardsError::from_message(400, "can't receive page"));
      }
  
      // verify here in order to avoid fetching the object twice over http
      object.verify(context).await?;
      object.receive(context).await
    }

}


impl Id for RawAnnouncableActivities {
    fn object_id(&self) -> &Url {
      ActivityHandler::id(self)
    }
  }
  
  impl TryFrom<RawAnnouncableActivities> for AnnouncableActivities {
    type Error = serde_json::error::Error;
  
    fn try_from(value: RawAnnouncableActivities) -> Result<Self, Self::Error> {
      let mut map = value.other.clone();
      map.insert("id".to_string(), Value::String(value.id.to_string()));
      map.insert("actor".to_string(), Value::String(value.actor.to_string()));
      serde_json::from_value(Value::Object(map))
    }
  }
  
  impl TryFrom<AnnouncableActivities> for RawAnnouncableActivities {
    type Error = serde_json::error::Error;
  
    fn try_from(value: AnnouncableActivities) -> Result<Self, Self::Error> {
      serde_json::from_value(serde_json::to_value(value)?)
    }
  }