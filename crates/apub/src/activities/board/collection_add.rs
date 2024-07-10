use crate::{
    activities::{
        board::send_activity_in_board, generate_activity_id, verify_is_public, verify_mod_action,
        verify_person_in_board,
    },
    activity_lists::AnnouncableActivities,
    insert_activity,
    objects::{board::ApubBoard, person::ApubPerson, post::ApubPost},
    protocol::{
        activities::board::{collection_add::CollectionAdd, collection_remove::CollectionRemove},
        InBoard,
    },
    SendActivity,
};
use tinyboards_api_common::{
    board::{AddBoardMod, BoardIdPath, BoardModResponse},
    data::TinyBoardsContext,
    post::{PostIdPath, PostResponse, TogglePostFeatured},
    utils::{generate_featured_url, generate_moderators_url, require_user},
};
use tinyboards_db::{
    impls::board::boards::CollectionType,
    models::{
        board::board_mods::{BoardModerator, BoardModeratorForm, ModPerms},
        board::boards::Board,
        moderator::mod_actions::{ModAddBoardMod, ModAddBoardModForm},
        person::person::Person,
        post::posts::Post,
    },
    traits::{Crud, Joinable},
};
use tinyboards_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::AddType, public},
    traits::{ActivityHandler, Actor},
};
use tinyboards_utils::error::TinyBoardsError;
use url::Url;

impl CollectionAdd {
    #[tracing::instrument(skip_all)]
    pub async fn send_add_mod(
        board: &ApubBoard,
        added_mod: &ApubPerson,
        actor: &ApubPerson,
        context: &Data<TinyBoardsContext>,
    ) -> Result<(), TinyBoardsError> {
        let id = generate_activity_id(
            AddType::Add,
            &context.settings().get_protocol_and_hostname(),
        )?;
        let add = CollectionAdd {
            actor: actor.id().into(),
            to: vec![public()],
            object: added_mod.id(),
            target: generate_moderators_url(&board.actor_id)?.into(),
            cc: vec![board.id()],
            kind: AddType::Add,
            id: id.clone(),
            audience: Some(board.id().into()),
        };
        let activity = AnnouncableActivities::CollectionAdd(add);
        let inboxes = vec![added_mod.shared_inbox_or_inbox()];
        send_activity_in_board(activity, actor, board, inboxes, true, context).await
    }

    pub async fn send_add_featured_post(
        board: &ApubBoard,
        featured_post: &ApubPost,
        actor: &ApubPerson,
        context: &Data<TinyBoardsContext>,
    ) -> Result<(), TinyBoardsError> {
        let id = generate_activity_id(
            AddType::Add,
            &context.settings().get_protocol_and_hostname(),
        )?;
        let add = CollectionAdd {
            actor: actor.id().into(),
            to: vec![public()],
            object: featured_post.ap_id.clone().unwrap().into(),
            target: generate_featured_url(&board.actor_id)?.into(),
            cc: vec![board.id()],
            kind: AddType::Add,
            id: id.clone(),
            audience: Some(board.id().into()),
        };
        let activity = AnnouncableActivities::CollectionAdd(add);
        send_activity_in_board(activity, actor, board, vec![], true, context).await
    }
}

#[async_trait::async_trait]
impl ActivityHandler for CollectionAdd {
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
        verify_mod_action(&self.actor, &self.object, board.id, context).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
        insert_activity(&self.id, &self, false, false, context).await?;
        let (board, collection_type) =
            Board::get_by_collection_url(context.pool(), &self.target.into()).await?;
        match collection_type {
            CollectionType::Moderators => {
                let new_mod = ObjectId::<ApubPerson>::from(self.object)
                    .dereference(context)
                    .await?;

                let new_mod_id = new_mod.id;
                let moderated_boards =
                    BoardModerator::get_person_moderated_boards(context.pool(), new_mod_id).await?;
                if !moderated_boards.contains(&board.id) {
                    let form = BoardModeratorForm {
                        board_id: Some(board.id),
                        person_id: Some(new_mod.id),
                        rank: Some(1),
                        invite_accepted: Some(true),
                        permissions: Some(ModPerms::Full.as_i32()),
                        invite_accepted_date: None,
                    };
                    BoardModerator::join(context.pool(), &form).await?;
                }
                // mod log
                let actor = self.actor.dereference(context).await?;
                let form = ModAddBoardModForm {
                    mod_person_id: actor.id,
                    other_person_id: new_mod.id,
                    board_id: board.id,
                    removed: Some(Some(false)),
                };
                ModAddBoardMod::create(context.pool(), &form).await?;
                // TODO: send notification locally about added mod
            }
            CollectionType::Featured => {
                let post = ObjectId::<ApubPost>::from(self.object)
                    .dereference(context)
                    .await?;
                Post::update_featured_board(context.pool(), post.id, true).await?;
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for AddBoardMod {
    type Response = BoardModResponse;
    type Route = BoardIdPath;

    async fn send_activity(
        request: &Self,
        _response: &Self::Response,
        context: &Data<TinyBoardsContext>,
        BoardIdPath { board_id }: &Self::Route,
        auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;
        let board: ApubBoard = Board::read(context.pool(), *board_id).await?.into();
        let updated_mod: ApubPerson =
            Person::read(context.pool(), request.person_id.unwrap_or(view.person.id))
                .await?
                .into();

        CollectionAdd::send_add_mod(&board, &updated_mod, &view.person.into(), context).await
    }
}

#[async_trait::async_trait]
impl SendActivity for TogglePostFeatured {
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
        let board = Board::read(context.pool(), response.post_view.board.id)
            .await?
            .into();
        let post = response.post_view.post.clone().into();
        let person: ApubPerson = view.person.into();
        if request.value {
            CollectionAdd::send_add_featured_post(&board, &post, &person, context).await?;
        } else {
            CollectionRemove::send_remove_featured_post(&board, &post, &person, context).await?;
        }
        Ok(())
    }
}
