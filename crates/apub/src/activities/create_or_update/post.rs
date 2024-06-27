use crate::{
    activities::{
        board::send_activity_in_board, check_board_deleted_or_removed, generate_activity_id,
        verify_is_public, verify_mod_action, verify_person_in_board,
    },
    activity_lists::AnnouncableActivities,
    insert_activity,
    objects::{board::ApubBoard, person::ApubPerson, post::ApubPost},
    protocol::{
        activities::{create_or_update::page::CreateOrUpdatePage, CreateOrUpdateType},
        InBoard,
    },
    SendActivity,
};
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{EditPost, PostResponse, SubmitPost},
};
use tinyboards_db::{
    models::{
        board::boards::Board,
        person::person::Person,
        post::post_votes::{PostVote, PostVoteForm},
        post::posts::Post,
    },
    traits::{Crud, Voteable},
};
use tinyboards_federation::{
    config::Data,
    kinds::public,
    protocol::verification::{verify_domains_match, verify_urls_match},
    traits::{ActivityHandler, Actor, Object},
};
use tinyboards_utils::error::TinyBoardsError;
use url::Url;

#[async_trait::async_trait]
impl SendActivity for SubmitPost {
    type Response = PostResponse;
    type Route = ();

    async fn send_activity(
        _request: &Self,
        response: &Self::Response,
        context: &Data<TinyBoardsContext>,
        _: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        CreateOrUpdatePage::send(
            &response.post_view.post,
            response.post_view.creator.clone().unwrap().id,
            CreateOrUpdateType::Create,
            context,
        )
        .await
    }
}

#[async_trait::async_trait]
impl SendActivity for EditPost {
    type Response = PostResponse;
    type Route = ();

    async fn send_activity(
        _request: &Self,
        response: &Self::Response,
        context: &Data<TinyBoardsContext>,
        _: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        CreateOrUpdatePage::send(
            &response.post_view.post,
            response.post_view.creator.clone().unwrap().id,
            CreateOrUpdateType::Update,
            context,
        )
        .await
    }
}

impl CreateOrUpdatePage {
    pub(crate) async fn new(
        post: ApubPost,
        actor: &ApubPerson,
        board: &ApubBoard,
        kind: CreateOrUpdateType,
        context: &Data<TinyBoardsContext>,
    ) -> Result<CreateOrUpdatePage, TinyBoardsError> {
        let id = generate_activity_id(
            kind.clone(),
            &context.settings().get_protocol_and_hostname(),
        )?;
        Ok(CreateOrUpdatePage {
            actor: actor.id().into(),
            to: vec![public()],
            object: post.into_json(context).await?,
            cc: vec![board.id()],
            kind,
            id: id.clone(),
            audience: Some(board.id().into()),
        })
    }

    #[tracing::instrument(skip_all)]
    pub(crate) async fn send(
        post: &Post,
        person_id: i32,
        kind: CreateOrUpdateType,
        context: &Data<TinyBoardsContext>,
    ) -> Result<(), TinyBoardsError> {
        let post = ApubPost(post.clone());
        let board_id = post.board_id;
        let person: ApubPerson = Person::read(context.pool(), person_id).await?.into();
        let board: ApubBoard = Board::read(context.pool(), board_id).await?.into();

        let create_or_update =
            CreateOrUpdatePage::new(post, &person, &board, kind, context).await?;
        let is_mod_action = create_or_update.object.is_mod_action(context).await?;
        let activity = AnnouncableActivities::CreateOrUpdatePost(create_or_update);
        send_activity_in_board(activity, &person, &board, vec![], is_mod_action, context).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for CreateOrUpdatePage {
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
        let board = self.board(context).await?;
        verify_person_in_board(&self.actor, &board, context).await?;
        check_board_deleted_or_removed(&board)?;

        match self.kind {
            CreateOrUpdateType::Create => {
                verify_domains_match(self.actor.inner(), self.object.id.inner())?;
                verify_urls_match(self.actor.inner(), self.object.creator()?.inner())?;
                // Check that the post isnt locked, as that isnt possible for newly created posts.
                // However, when fetching a remote post we generate a new create activity with the current
                // locked value, so this check may fail. So only check if its a local board,
                // because then we will definitely receive all create and update activities separately.
                let is_locked = self.object.comments_enabled == Some(false);
                if board.local && is_locked {
                    return Err(TinyBoardsError::from_message(
                        400,
                        "new post cannot be locked",
                    ));
                }
            }
            CreateOrUpdateType::Update => {
                let is_mod_action = self.object.is_mod_action(context).await?;
                if is_mod_action {
                    verify_mod_action(&self.actor, self.object.id.inner(), board.id, context)
                        .await?;
                } else {
                    verify_domains_match(self.actor.inner(), self.object.id.inner())?;
                    verify_urls_match(self.actor.inner(), self.object.creator()?.inner())?;
                }
            }
        }
        ApubPost::verify(&self.object, self.actor.inner(), context).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn receive(self, context: &Data<TinyBoardsContext>) -> Result<(), TinyBoardsError> {
        insert_activity(&self.id, &self, false, false, context).await?;
        let post = ApubPost::from_json(self.object, context).await?;

        // author likes their own post by default
        let like_form = PostVoteForm {
            post_id: post.id,
            person_id: post.creator_id,
            score: 1,
        };
        PostVote::vote(context.pool(), &like_form).await?;
        Ok(())
    }
}
