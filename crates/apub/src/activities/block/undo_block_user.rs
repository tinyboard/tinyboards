use crate::{
    activities::{
      block::{generate_cc, SiteOrBoard},
      board::send_activity_in_board,
      generate_activity_id,
      send_tinyboards_activity,
      verify_is_public,
    },
    activity_lists::AnnouncableActivities,
    insert_activity,
    objects::{instance::remote_instance_inboxes, person::ApubPerson},
    protocol::activities::block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
  };
  use tinyboards_federation::{
    config::Data,
    kinds::{activity::UndoType, public},
    protocol::verification::verify_domains_match,
    traits::{ActivityHandler, Actor},
  };
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_db::{
    models::{
        board::board_person_bans::{BoardPersonBan, BoardPersonBanForm},
        moderator::mod_actions::{ModBan, ModBanForm, ModBanFromBoard, ModBanFromBoardForm},
        person::person::{Person, PersonForm},
    },
    traits::{Bannable, Crud},
  };
  use tinyboards_utils::error::TinyBoardsError;
  use url::Url;

  impl UndoBlockUser {
    #[tracing::instrument(skip_all)]
    pub async fn send(
      target: &SiteOrBoard,
      user: &ApubPerson,
      mod_: &ApubPerson,
      reason: Option<String>,
      context: &Data<TinyBoardsContext>,
    ) -> Result<(), TinyBoardsError> {
      let block = BlockUser::new(target, user, mod_, None, reason, None, context).await?;
      let audience = if let SiteOrBoard::Board(b) = target {
        Some(b.id().into())
      } else {
        None
      };
  
      let id = generate_activity_id(
        UndoType::Undo,
        &context.settings().get_protocol_and_hostname(),
      )?;
      let undo = UndoBlockUser {
        actor: mod_.id().into(),
        to: vec![public()],
        object: block,
        cc: generate_cc(target, context.pool()).await?,
        kind: UndoType::Undo,
        id: id.clone(),
        audience,
      };
  
      let mut inboxes = vec![user.shared_inbox_or_inbox()];
      match target {
        SiteOrBoard::Site(_) => {
          inboxes.append(&mut remote_instance_inboxes(context.pool()).await?);
          send_tinyboards_activity(context, undo, mod_, inboxes, false).await
        }
        SiteOrBoard::Board(b) => {
          let activity = AnnouncableActivities::UndoBlockUser(undo);
          send_activity_in_board(activity, mod_, b, inboxes, true, context).await
        }
      }
    }
  }

#[async_trait::async_trait]
impl ActivityHandler for UndoBlockUser {
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
    verify_is_public(&self.to, &self.cc)?;
    verify_domains_match(self.actor.inner(), self.object.actor.inner())?;
    self.object.verify(context).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<TinyBoardsContext>) -> Result<(), TinyBoardsError> {
    insert_activity(&self.id, &self, false, false, context).await?;
    let expires = Some(self.object.expires.map(|u| u.naive_local()));
    let mod_person = self.actor.dereference(context).await?;
    let blocked_person = self.object.object.dereference(context).await?;
    match self.object.target.dereference(context).await? {
        SiteOrBoard::Site(_site) => {
        let blocked_person = Person::update(
          context.pool(),
          blocked_person.id,
          &PersonForm::builder()
            .banned(Some(false))
            .ban_expires(Some(expires))
            .build(),
        )
        .await?;

        // write mod log
        let form = ModBanForm {
          mod_person_id: mod_person.id,
          other_person_id: blocked_person.id,
          reason: Some(self.object.summary),
          banned: Some(Some(false)),
          expires,
        };
        ModBan::create(context.pool(), &form).await?;
      }
      SiteOrBoard::Board(board) => {
        let board_person_ban_form = BoardPersonBanForm {
          board_id: board.id,
          person_id: blocked_person.id,
          expires: None,
        };
        BoardPersonBan::unban(context.pool(), &board_person_ban_form).await?;

        // write to mod log
        let form = ModBanFromBoardForm {
          mod_person_id: mod_person.id,
          other_person_id: blocked_person.id,
          board_id: board.id,
          reason: Some(self.object.summary),
          banned: Some(Some(false)),
          expires,
        };
        ModBanFromBoard::create(context.pool(), &form).await?;
      }
    }

    Ok(())
  }
}