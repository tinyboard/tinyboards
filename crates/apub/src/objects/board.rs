use crate::{
    check_ap_id_valid_with_strictness,
    fetch_local_site_data,
    objects::instance::fetch_instance_actor_for_object,
    protocol::{
        objects::{group::Group, Endpoints, LanguageTag},
        ImageObject,
        Source,
    },
};
use tinyboards_federation::{
    config::Data,
    kinds::actor::GroupType,
    traits::{Actor, Object},
};
use chrono::NaiveDateTime;
use itertools::Itertools;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    utils::{generate_featured_url, generate_moderators_url, generate_outbox_url},
};
use tinyboards_db::{
    models::{
        apub::actor_language::BoardLanguage,
        board::boards::{Board},
    },
    traits::{ApubActor, Crud},
};
use tinyboards_db_views::structs::BoardSubscriberView;
use tinyboards_utils::{
    error::TinyBoardsError,
    time::convert_datetime,
    parser::parse_markdown,
};
use std::ops::Deref;
use tracing::debug;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubBoard(Board);

impl Deref for ApubBoard {
    type Target = Board;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Board> for ApubBoard {
    fn from(b: Board) -> Self {
        ApubBoard(b)
    }
}

#[async_trait::async_trait]
impl Object for ApubBoard {
    type DataType = TinyBoardsContext;
    type Kind = Group;
    type Error = TinyBoardsError;

    fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
        Some(self.last_refreshed_date)
    }

    #[tracing::instrument(skip_all)]
    async fn read_from_id(
        object_id: Url,
        context: &Data<Self::DataType>,
    ) -> Result<Option<Self>, TinyBoardsError> {
        Ok(
            Board::read_from_apub_id(context.pool(), &object_id.into())
                .await?
                .map(Into::into)
        )
    }

    #[tracing::instrument(skip_all)]
    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Group, TinyBoardsError> {
        let board_id = self.id;
        let langs = BoardLanguage::read(data.pool(), board_id).await?;
        let language = LanguageTag::new_multiple(langs, data.pool()).await?;

        let group = Group {
            kind: GroupType::Group,
            id: self.id().into(),
            preferred_username: self.name.clone(),
            name: Some(self.title.clone()),
            summary: self.description.as_ref().map(|b| parse_markdown(b)).unwrap(),
            source: self.description.clone().map(Source::new),
            icon: self.icon.clone().map(ImageObject::new),
            image: self.banner.clone().map(ImageObject::new),
            sensitive: Some(self.is_nsfw),
            featured: Some(generate_featured_url(&self.actor_id)?.into()),
            inbox: self.inbox_url.clone().into(),
            outbox: generate_outbox_url(&self.actor_id)?.into(),
            followers: self.subscribers_url.clone().into(),
            endpoints: self.shared_inbox_url.clone().map(|s|
                 Endpoints { shared_inbox: s.into() }
            ),
            public_key: self.public_key(),
            language,
            published: Some(convert_datetime(self.creation_date)),
            updated: self.updated.map(convert_datetime),
            posting_restricted_to_mods: Some(self.posting_restricted_to_mods),
            attributed_to: Some(generate_moderators_url(&self.actor_id)?.into()),
        };

        Ok(group)
    }

    #[tracing::instrument(skip_all)]
    async fn verify(
        group: &Group,
        expected_domain: &Url,
        context: &Data<Self::DataType>,
    ) -> Result<(), TinyBoardsError> {
        group.verify(expected_domain, context).await
    }

    #[tracing::instrument(skip_all)]
    async fn from_json(
        group: Group,
        context: &Data<Self::DataType>, 
    ) -> Result<ApubBoard, TinyBoardsError> {
        let instance_id = fetch_instance_actor_for_object(&group.id, context).await?;

        let form = Group::into_form(group.clone(), instance_id);

        let languages = LanguageTag::to_language_id_multiple(group.language, context.pool()).await?;

        let board = Board::create(context.pool(), &form).await?;
        BoardLanguage::update(context.pool(), languages, board.id).await?;

        let board: ApubBoard = board.into();

        group
            .outbox
            .dereference(&board, context)
            .await
            .map_err(|e| debug!("{}", e))
            .ok();

        if let Some(moderators) = group.attributed_to {
            moderators
                .dereference(&board, context)
                .await
                .map_err(|e| debug!("{}", e))
                .ok();
        }

        Ok(board)
    }
}

impl Actor for ApubBoard {
    fn id(&self) -> Url {
        self.actor_id.inner().clone()
      }
    
      fn public_key_pem(&self) -> &str {
        &self.public_key
      }
    
      fn private_key_pem(&self) -> Option<String> {
        self.private_key.clone()
      }
    
      fn inbox(&self) -> Url {
        self.inbox_url.clone().into()
      }
    
      fn shared_inbox(&self) -> Option<Url> {
        self.shared_inbox_url.clone().map(Into::into)
      }
}

impl ApubBoard {
    #[tracing::instrument(skip_all)]
    pub(crate) async fn get_subscriber_inboxes(
      &self,
      context: &TinyBoardsContext,
    ) -> Result<Vec<Url>, TinyBoardsError> {
      let id = self.id;
  
      let local_site_data = fetch_local_site_data(context.pool()).await?;
      let follows = BoardSubscriberView::for_board(context.pool(), id).await?;
      let inboxes: Vec<Url> = follows
        .into_iter()
        .filter(|s| !s.subscriber.local)
        .map(|s| {
          s.subscriber
            .shared_inbox_url
            .unwrap_or(s.subscriber.inbox_url)
            .into()
        })
        .unique()
        .filter(|inbox: &Url| inbox.host_str() != Some(&context.settings().hostname))
        // Don't send to blocked instances
        .filter(|inbox| {
          check_ap_id_valid_with_strictness(inbox, false, &local_site_data, context.settings())
            .is_ok()
        })
        .collect();
  
      Ok(inboxes)
    }
}