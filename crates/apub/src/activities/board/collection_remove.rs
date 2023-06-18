use crate::{
    activities::{
      board::send_activity_in_board,
      generate_activity_id,
      verify_is_public,
      verify_mod_action,
      verify_person_in_board,
    },
    activity_lists::AnnouncableActivities,
    insert_activity,
    objects::{board::ApubBoard, person::ApubPerson, post::ApubPost},
    protocol::{activities::board::collection_remove::CollectionRemove},
  };
  use tinyboards_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::RemoveType, public},
    traits::{ActivityHandler, Actor},
  };
  use tinyboards_api_common::{
    data::TinyBoardsContext,
    utils::{generate_featured_url, generate_moderators_url},
  };
  use tinyboards_db::{
    impls::board::boards::CollectionType,
    models::{
        board::boards::Board,
        board::board_mods::{BoardModerator, BoardModeratorForm},
        moderator::mod_actions::{ModAddBoardMod, ModAddBoardModForm},
        post::posts::{Post},
    },
    traits::{Crud, Joinable},
  };
  use tinyboards_utils::error::TinyBoardsError;
  use url::Url;
  
  impl CollectionRemove {
    #[tracing::instrument(skip_all)]
    pub async fn send_remove_mod(
      board: &ApubBoard,
      removed_mod: &ApubPerson,
      actor: &ApubPerson,
      context: &Data<TinyBoardsContext>,
    ) -> Result<(), TinyBoardsError> {
      let id = generate_activity_id(
        RemoveType::Remove,
        &context.settings().get_protocol_and_hostname(),
      )?;
      let remove = CollectionRemove {
        actor: actor.id().into(),
        to: vec![public()],
        object: removed_mod.id(),
        target: generate_moderators_url(&board.actor_id)?.into(),
        id: id.clone(),
        cc: vec![board.id()],
        kind: RemoveType::Remove,
        audience: Some(board.id().into()),
      };
  
      let activity = AnnouncableActivities::CollectionRemove(remove);
      let inboxes = vec![removed_mod.shared_inbox_or_inbox()];
      send_activity_in_board(activity, actor, board, inboxes, true, context).await
    }
  
    pub async fn send_remove_featured_post(
      board: &ApubBoard,
      featured_post: &ApubPost,
      actor: &ApubPerson,
      context: &Data<TinyBoardsContext>,
    ) -> Result<(), TinyBoardsError> {
      let id = generate_activity_id(
        RemoveType::Remove,
        &context.settings().get_protocol_and_hostname(),
      )?;
      let remove = CollectionRemove {
        actor: actor.id().into(),
        to: vec![public()],
        object: featured_post.ap_id.clone().into(),
        target: generate_featured_url(&board.actor_id)?.into(),
        cc: vec![board.id()],
        kind: RemoveType::Remove,
        id: id.clone(),
        audience: Some(board.id().into()),
      };
      let activity = AnnouncableActivities::CollectionRemove(remove);
      send_activity_in_board(activity, actor, board, vec![], true, context).await
    }
  }
  
  #[async_trait::async_trait]
  impl ActivityHandler for CollectionRemove {
    type DataType = TinyBoardsContext;
    type Error = TinyBoardsError;
  
    fn id(&self) -> &Url {
      &self.id
    }
  
    fn actor(&self) -> &Url {
      self.actor.inner()
    }
  
    #[tracing::instrument(skip_all)]
    async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
      verify_is_public(&self.to, &self.cc)?;
      let community = self.community(context).await?;
      verify_person_in_board(&self.actor, &community, context).await?;
      verify_mod_action(&self.actor, &self.object, community.id, context).await?;
      Ok(())
    }
  
    #[tracing::instrument(skip_all)]
    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
      insert_activity(&self.id, &self, false, false, context).await?;
      let (community, collection_type) =
        Board::get_by_collection_url(context.pool(), &self.target.into()).await?;
      match collection_type {
        CollectionType::Moderators => {
          let remove_mod = ObjectId::<ApubPerson>::from(self.object)
            .dereference(context)
            .await?;
  
          let form = BoardModeratorForm {
            board_id: community.id,
            person_id: remove_mod.id,
          };
          BoardModerator::leave(context.pool(), &form).await?;
  
          // write mod log
          let actor = self.actor.dereference(context).await?;
          let form = ModAddBoardModForm {
            mod_person_id: actor.id,
            other_person_id: remove_mod.id,
            board_id: community.id,
            removed: Some(Some(true)),
          };
          ModAddBoardMod::create(context.pool(), &form).await?;
  
          // TODO: send websocket notification about removed mod
        }
        CollectionType::Featured => {
          let post = ObjectId::<ApubPost>::from(self.object)
            .dereference(context)
            .await?;
          Post::update_featured_board(context.pool(), post.id, false);
        }
      }
      Ok(())
    }
  }