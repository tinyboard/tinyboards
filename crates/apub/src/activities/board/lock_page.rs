use crate::{
    activities::{
        board::send_activity_in_board, check_board_deleted_or_removed, generate_activity_id,
        verify_is_public, verify_mod_action, verify_person_in_board,
    },
    activity_lists::AnnouncableActivities,
    insert_activity,
    protocol::{
        activities::board::lock_page::{LockPage, LockType, UndoLockPage},
        InBoard,
    },
    SendActivity,
};
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{PostIdPath, PostResponse, TogglePostLock},
    utils::require_user,
};
use tinyboards_db::{
    models::{
        board::boards::Board,
        post::posts::{Post, PostForm},
    },
    traits::Crud,
    utils::naive_now,
};
use tinyboards_federation::{
    config::Data,
    kinds::{activity::UndoType, public},
    traits::ActivityHandler,
};
use tinyboards_utils::error::TinyBoardsError;
use url::Url;

#[async_trait::async_trait]
impl ActivityHandler for LockPage {
    type DataType = TinyBoardsContext;
    type Error = TinyBoardsError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_is_public(&self.to, &self.cc)?;
        let board = self.board(context).await?;
        verify_person_in_board(&self.actor, &board, context).await?;
        check_board_deleted_or_removed(&board)?;
        verify_mod_action(&self.actor, self.object.inner(), board.id, context).await?;
        Ok(())
    }

    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let form = PostForm {
            is_locked: Some(true),
            updated: Some(naive_now()),
            ..PostForm::default()
        };
        let post = self.object.dereference(context).await?;
        Post::update(context.pool(), post.id, &form).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for UndoLockPage {
    type DataType = TinyBoardsContext;
    type Error = TinyBoardsError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_is_public(&self.to, &self.cc)?;
        let board = self.board(context).await?;
        verify_person_in_board(&self.actor, &board, context).await?;
        check_board_deleted_or_removed(&board)?;
        verify_mod_action(&self.actor, self.object.object.inner(), board.id, context).await?;
        Ok(())
    }

    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        insert_activity(&self.id, &self, false, false, context).await?;
        let form = PostForm {
            is_locked: Some(false),
            updated: Some(naive_now()),
            ..PostForm::default()
        };
        let post = self.object.object.dereference(context).await?;
        Post::update(context.pool(), post.id, &form).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for TogglePostLock {
    type Response = PostResponse;
    type Route = PostIdPath;

    async fn send_activity(
        request: &Self,
        response: &Self::Response,
        context: &Data<TinyBoardsContext>,
        _: &Self::Route,
        auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;
        let id = generate_activity_id(
            LockType::Lock,
            &context.settings().get_protocol_and_hostname(),
        )?;
        let board_id = response.post_view.board.actor_id.clone();
        let actor = view.person.actor_id.clone().into();
        let lock = LockPage {
            actor,
            to: vec![public()],
            object: response.post_view.post.ap_id.clone().unwrap().into(),
            cc: vec![board_id.clone().into()],
            kind: LockType::Lock,
            id,
            audience: Some(board_id.into()),
        };
        let activity = if request.value {
            AnnouncableActivities::LockPost(lock)
        } else {
            let id = generate_activity_id(
                UndoType::Undo,
                &context.settings().get_protocol_and_hostname(),
            )?;
            let undo = UndoLockPage {
                actor: lock.actor.clone(),
                to: vec![public()],
                cc: lock.cc.clone(),
                kind: UndoType::Undo,
                id,
                audience: lock.audience.clone(),
                object: lock,
            };
            AnnouncableActivities::UndoLockPost(undo)
        };
        let board = Board::read(context.pool(), response.post_view.board.id).await?;
        send_activity_in_board(
            activity,
            &view.person.into(),
            &board.into(),
            vec![],
            true,
            context,
        )
        .await?;
        Ok(())
    }
}
