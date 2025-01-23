use crate::{
    activities::{
        block::{generate_cc, SiteOrBoard},
        board::send_activity_in_board,
        generate_activity_id,
        send_tinyboards_activity,
        verify_is_public,
        verify_mod_action,
        verify_person_in_board,
    },
    activity_lists::AnnouncableActivities,
    insert_activity,
    objects::{instance::remote_instance_inboxes, person::ApubPerson},
    protocol::activities::block::block_user::BlockUser,
};
use tinyboards_federation::{
    config::Data,
    kinds::{activity::BlockType, public},
    protocol::verification::verify_domains_match,
    traits::{ActivityHandler, Actor},
};
use anyhow::anyhow;
use chrono::NaiveDateTime;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    utils::{remove_user_data, remove_user_data_in_board},
};
use tinyboards_db::{
    models::{
        board::board_subscriber::{BoardSubscriber, BoardSubscriberForm},
        board::board_person_bans::{BoardPersonBan, BoardPersonBanForm},
        person::person::{Person, PersonForm}, moderator::mod_actions::{ModBanForm, ModBan, ModBanFromBoardForm, ModBanFromBoard},
    },
    traits::{Bannable, Crud, Subscribeable}, utils::naive_now,
};
use tinyboards_utils::{error::TinyBoardsError, time::convert_datetime};
use url::Url;

impl BlockUser {
    pub(in crate::activities::block) async fn new(
        target: &SiteOrBoard,
        user: &ApubPerson,
        mod_: &ApubPerson,
        remove_data: Option<bool>,
        reason: Option<String>,
        expires: Option<NaiveDateTime>,
        context: &Data<TinyBoardsContext>,
    ) -> Result<BlockUser, TinyBoardsError> {
        let audience = if let SiteOrBoard::Board(b) = target {
            Some(b.id().into())
        } else {
            None
        };
        Ok(BlockUser {
            actor: mod_.id().into(),
            to: vec![public()],
            object: user.id().into(),
            cc: generate_cc(target, context.pool()).await?,
            target: target.id(),
            kind: BlockType::Block,
            remove_data,
            summary: reason,
            id: generate_activity_id(
                BlockType::Block, 
                &context.settings().get_protocol_and_hostname(),
            )?,
            audience,
            expires: expires.map(convert_datetime)
        })
    }

    #[tracing::instrument(skip_all)]
    pub async fn send(
        target: &SiteOrBoard,
        user: &ApubPerson,
        mod_: &ApubPerson,
        remove_data: bool,
        reason: Option<String>,
        expires: Option<NaiveDateTime>,
        context: &Data<TinyBoardsContext>,
    ) -> Result<(), TinyBoardsError> {
        let block = BlockUser::new(
            target,
            user,
            mod_,
            Some(remove_data),
            reason,
            expires,
            context,
        ).await?;

        match target {
            SiteOrBoard::Site(_) => {
                let inboxes = remote_instance_inboxes(context.pool()).await?;
                send_tinyboards_activity(context, block, mod_, inboxes, false).await
            },
            SiteOrBoard::Board(b) => {
                let activity = AnnouncableActivities::BlockUser(block);
                let inboxes = vec![user.shared_inbox_or_inbox()];
                send_activity_in_board(activity, mod_, b, inboxes, true, context).await
            }
        }
    }
}

#[async_trait::async_trait]
impl ActivityHandler for BlockUser {
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
        match self.target.dereference(context).await? {
            SiteOrBoard::Site(site) => {
                let domain = self.object.inner().domain().expect("url needs domain");
                if context.settings().hostname == domain {
                    return Err(
                        anyhow!("site bans from remote instance can't affect user's home instance").into()
                    );
                }
                // site ban can only target a user who is on the same instance as the actor
                verify_domains_match(&site.id(), self.actor.inner())?;
                verify_domains_match(&site.id(), self.object.inner())?;
            },
            SiteOrBoard::Board(board) => {
                verify_person_in_board(&self.actor, &board, context).await?;
                verify_mod_action(&self.actor, self.object.inner(), board.id, context).await?;
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn receive(self, context: &Data<TinyBoardsContext>) -> Result<(), TinyBoardsError> {
        insert_activity(&self.id, &self, false, false, context).await?;
        let expires = self.expires.map(|u| u.naive_local());
        let mod_person = self.actor.dereference(context).await?;
        let blocked_person = self.object.dereference(context).await?;
        let target = self.target.dereference(context).await?;
        match target {
            SiteOrBoard::Site(_site) => {
                let blocked_form = PersonForm {
                    is_banned: Some(true),
                    unban_date: expires,
                    updated: Some(naive_now()),
                    ..PersonForm::default()
                };
                let blocked_person = Person::update(
                    context.pool(), 
                    blocked_person.id, 
                    &blocked_form
                )
                .await?;
                if self.remove_data.unwrap_or(false) {
                    remove_user_data(
                        blocked_person.id,
                        context.pool(),
                    )
                    .await?;
                }

                let form = ModBanForm {
                    mod_person_id: mod_person.id,
                    other_person_id: blocked_person.id,
                    reason: Some(self.summary),
                    banned: Some(Some(true)),
                    expires: Some(expires),
                };
                ModBan::create(context.pool(), &form).await?;
            },
            SiteOrBoard::Board(board) => {
                let board_user_ban_form = BoardPersonBanForm {
                    board_id: board.id,
                    person_id: blocked_person.id,
                    expires,
                };
                BoardPersonBan::ban(context.pool(), &board_user_ban_form).await?;

                let board_subscriber_form = BoardSubscriberForm {
                    board_id: board.id,
                    person_id: blocked_person.id,
                    pending: Some(false),
                };
                BoardSubscriber::unsubscribe(context.pool(), &board_subscriber_form).await.ok();

                if self.remove_data.unwrap_or(false) {
                    remove_user_data_in_board(board.id, blocked_person.id, context.pool()).await?;
                }

                let form = ModBanFromBoardForm {
                    mod_person_id: mod_person.id,
                    other_person_id: blocked_person.id,
                    board_id: board.id,
                    reason: Some(self.summary),
                    banned: Some(Some(true)),
                    expires: Some(expires),
                };
                ModBanFromBoard::create(context.pool(), &form).await?;
            }
        }
        Ok(())
    }
}