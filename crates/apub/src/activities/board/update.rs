use crate::{
    activities::{
        board::send_activity_in_board, generate_activity_id, verify_is_public, verify_mod_action,
        verify_person_in_board,
    },
    activity_lists::AnnouncableActivities,
    insert_activity,
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::{activities::board::update::UpdateBoard, InBoard},
    SendActivity,
};
use tinyboards_api_common::{
    board::{BoardResponse, EditBoard, HideBoard},
    data::TinyBoardsContext,
    utils::require_user,
};
use tinyboards_db::{models::board::boards::Board, traits::Crud};
use tinyboards_federation::{
    config::Data,
    kinds::{activity::UpdateType, public},
    traits::{ActivityHandler, Actor, Object},
};
use tinyboards_utils::error::TinyBoardsError;
use url::Url;

#[async_trait::async_trait]
impl SendActivity for EditBoard {
    type Response = BoardResponse;
    type Route = ();

    async fn send_activity(
        _request: &Self,
        response: &Self::Response,
        context: &Data<TinyBoardsContext>,
        _: &Self::Route,
        auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;
        let board = Board::read(context.pool(), response.board_view.board.id).await?;
        UpdateBoard::send(board.into(), &view.person.into(), context).await
    }
}

impl UpdateBoard {
    #[tracing::instrument(skip_all)]
    pub async fn send(
        board: ApubBoard,
        actor: &ApubPerson,
        context: &Data<TinyBoardsContext>,
    ) -> Result<(), TinyBoardsError> {
        let id = generate_activity_id(
            UpdateType::Update,
            &context.settings().get_protocol_and_hostname(),
        )?;
        let update = UpdateBoard {
            actor: actor.id().into(),
            to: vec![public()],
            object: Box::new(board.clone().into_json(context).await?),
            cc: vec![board.id()],
            kind: UpdateType::Update,
            id: id.clone(),
            audience: Some(board.id().into()),
        };

        let activity = AnnouncableActivities::UpdateBoard(update);
        send_activity_in_board(activity, actor, &board, vec![], true, context).await
    }
}

#[async_trait::async_trait]
impl ActivityHandler for UpdateBoard {
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
        let board = self.board(context).await?;
        verify_person_in_board(&self.actor, &board, context).await?;
        verify_mod_action(&self.actor, self.object.id.inner(), board.id, context).await?;
        ApubBoard::verify(&self.object, &board.actor_id.clone().into(), context).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
        insert_activity(&self.id, &self, false, false, context).await?;
        let board = self.board(context).await?;

        let board_update_form = self.object.into_form(board.instance_id);

        Board::update(context.pool(), board.id, &board_update_form).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for HideBoard {
    type Response = BoardResponse;
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
        UpdateBoard::send(board.into(), &view.person.into(), context).await
    }
}
