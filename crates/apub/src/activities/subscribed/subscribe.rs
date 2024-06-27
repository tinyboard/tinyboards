use crate::{
    activities::{
        generate_activity_id, send_tinyboards_activity, verify_person, verify_person_in_board,
    },
    fetcher::user_or_board::UserOrBoard,
    insert_activity,
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::activities::subscribed::{
        accept::AcceptSubscribe, subscribe::Subscribe, undo_subscribe::UndoSubscribe,
    },
    SendActivity,
};
use tinyboards_api_common::{
    board::{BlockBoard, BlockBoardResponse},
    data::TinyBoardsContext,
    utils::require_user,
};
use tinyboards_db::{
    models::{
        board::board_subscriber::{BoardSubscriber, BoardSubscriberForm},
        board::boards::Board,
        person::person_subscriber::{PersonSubscriber, PersonSubscriberForm},
    },
    traits::{Crud, Subscribeable},
};
use tinyboards_federation::{
    config::Data,
    kinds::activity::FollowType,
    protocol::verification::verify_urls_match,
    traits::{ActivityHandler, Actor},
};
use tinyboards_utils::error::TinyBoardsError;
use url::Url;

impl Subscribe {
    pub(in crate::activities::subscribed) fn new(
        actor: &ApubPerson,
        board: &ApubBoard,
        context: &Data<TinyBoardsContext>,
    ) -> Result<Subscribe, TinyBoardsError> {
        Ok(Subscribe {
            actor: actor.id().into(),
            object: board.id().into(),
            to: Some([board.id().into()]),
            kind: FollowType::Follow,
            id: generate_activity_id(
                FollowType::Follow,
                &context.settings().get_protocol_and_hostname(),
            )?,
        })
    }

    #[tracing::instrument(skip_all)]
    pub async fn send(
        actor: &ApubPerson,
        board: &ApubBoard,
        context: &Data<TinyBoardsContext>,
    ) -> Result<(), TinyBoardsError> {
        let board_subscriber_form = BoardSubscriberForm {
            board_id: board.id,
            person_id: actor.id,
            pending: Some(true),
        };
        BoardSubscriber::subscribe(context.pool(), &board_subscriber_form)
            .await
            .ok();

        let follow = Subscribe::new(actor, board, context)?;
        let inbox = vec![board.shared_inbox_or_inbox()];
        send_tinyboards_activity(context, follow, actor, inbox, true).await
    }
}

#[async_trait::async_trait]
impl ActivityHandler for Subscribe {
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
        verify_person(&self.actor, context).await?;
        let object = self.object.dereference(context).await?;
        if let UserOrBoard::Board(b) = object {
            verify_person_in_board(&self.actor, &b, context).await?;
        }
        if let Some(to) = &self.to {
            verify_urls_match(to[0].inner(), self.object.inner())?;
        }
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn receive(self, context: &Data<TinyBoardsContext>) -> Result<(), TinyBoardsError> {
        insert_activity(&self.id, &self, false, true, context).await?;
        let actor = self.actor.dereference(context).await?;
        let object = self.object.dereference(context).await?;
        match object {
            UserOrBoard::User(u) => {
                let form = PersonSubscriberForm {
                    person_id: u.id,
                    subscriber_id: actor.id,
                    pending: false,
                };
                PersonSubscriber::subscribe(context.pool(), &form).await?;
            }
            UserOrBoard::Board(b) => {
                let form = BoardSubscriberForm {
                    board_id: b.id,
                    person_id: actor.id,
                    pending: Some(false),
                };
                BoardSubscriber::subscribe(context.pool(), &form).await?;
            }
        }

        AcceptSubscribe::send(self, context).await
    }
}

#[async_trait::async_trait]
impl SendActivity for BlockBoard {
    type Response = BlockBoardResponse;
    type Route = ();

    async fn send_activity(
        request: &Self,
        _response: &Self::Response,
        context: &Data<TinyBoardsContext>,
        _: &Self::Route,
        auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;
        let board = Board::read(context.pool(), request.board_id).await?;
        UndoSubscribe::send(&view.person.into(), &board.into(), context).await
    }
}
