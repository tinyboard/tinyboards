use crate::{
    objects::{board::ApubBoard, instance::ApubSite, person::ApubPerson},
    protocol::{
        activities::block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
        objects::{group::Group, instance::Instance},
    },
    SendActivity,
};
use tinyboards_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    traits::{Actor, Object},
};
use chrono::NaiveDateTime;
use tinyboards_api_common::{
    moderator::{BanFromBoard, BanFromBoardResponse},
    utils::require_user, data::TinyBoardsContext, person::{BanPerson, BanPersonResponse},
};
use tinyboards_db::{
    models::{board::boards::Board, person::person::Person, site::site::Site},
    traits::Crud,
    utils::DbPool,
};
use tinyboards_db_views::structs::SiteView;
use tinyboards_utils::{error::TinyBoardsError, time::naive_from_unix};
use serde::Deserialize;
use url::Url;

pub mod block_user;
pub mod undo_block_user;

#[derive(Clone, Debug)]
pub enum SiteOrBoard {
    Site(ApubSite),
    Board(ApubBoard),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum InstanceOrGroup {
    Instance(Instance),
    Group(Group),
}

#[async_trait::async_trait]
impl Object for SiteOrBoard {
    type DataType = TinyBoardsContext;
    type Kind = InstanceOrGroup;
    type Error = TinyBoardsError;

    #[tracing::instrument(skip_all)]
    fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
        Some(match self {
            SiteOrBoard::Site(i) => i.last_refreshed_date,
            SiteOrBoard::Board(g) => g.last_refreshed_date,
        })
    }

    #[tracing::instrument(skip_all)]
    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, TinyBoardsError> 
    where
        Self: Sized,
    {
        let site = ApubSite::read_from_id(object_id.clone(), data).await?;
        Ok(match site {
            Some(s) => Some(SiteOrBoard::Site(s)),
            None => ApubBoard::read_from_id(object_id, data)
                .await?
                .map(SiteOrBoard::Board)
        })
    }

    #[tracing::instrument(skip_all)]
    async fn delete(self, _data: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
        unimplemented!()
    }

    #[tracing::instrument(skip_all)]
    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, TinyBoardsError> {
        unimplemented!()
    }

    #[tracing::instrument(skip_all)]
    async fn verify(
        apub: &Self::Kind,
        expected_domain: &Url,
        data: &Data<Self::DataType>
    ) -> Result<(), TinyBoardsError> {
        match apub {
            InstanceOrGroup::Instance(s) => ApubSite::verify(s, expected_domain, data).await,
            InstanceOrGroup::Group(b) => ApubBoard::verify(b, expected_domain, data).await,
        }
    }

    #[tracing::instrument(skip_all)]
    async fn from_json(apub: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, TinyBoardsError> 
    where
        Self: Sized,
    {
        Ok(match apub {
            InstanceOrGroup::Instance(s) => SiteOrBoard::Site(ApubSite::from_json(s, data).await?),
            InstanceOrGroup::Group(b) => SiteOrBoard::Board(ApubBoard::from_json(b, data).await?)
        })
    }
}

impl SiteOrBoard {
    fn id(&self) -> ObjectId<SiteOrBoard> {
        match self {
            SiteOrBoard::Site(s) => ObjectId::from(s.actor_id.clone()),
            SiteOrBoard::Board(b) => ObjectId::from(b.actor_id.clone()),
        }
    }
}

async fn generate_cc(target: &SiteOrBoard, pool: &DbPool) -> Result<Vec<Url>, TinyBoardsError> {
    Ok(match target {
        SiteOrBoard::Site(_) => Site::read_remote_sites(pool)
            .await?
            .into_iter()
            .map(|s| s.actor_id.into())
            .collect(),
        SiteOrBoard::Board(b) => vec![b.id()],
    })
}

#[async_trait::async_trait]
impl SendActivity for BanPerson {
    type Response = BanPersonResponse;
    async fn send_activity(
        request: &Self,
        _response: &Self::Response,
        context: &Data<TinyBoardsContext>,
        auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        let local_user_view = require_user(context.pool(), context.master_key(), auth).await.unwrap()?;
        let person = Person::read(context.pool(), request.person_id).await?;
        let site = SiteOrBoard::Site(SiteView::read_local(context.pool()).await?.site.into());
        let expires = request.expires.map(naive_from_unix);

        // if the action affects a local user, federate to other instances
        if person.local {
            if request.ban {
                BlockUser::send(
                    &site,
                    &person.into(),
                    &local_user_view.person.into(),
                    request.remove_data.unwrap_or(false),
                    request.reason.clone(),
                    expires,
                    context,
                )
                .await
            } else {
                UndoBlockUser::send(
                    &site,
                    &person.into(),
                    &local_user_view.person.into(),
                    request.reason.clone(),
                    context,
                )
                .await
            }
        } else {
            Ok(())
        }
    }
}


#[async_trait::async_trait]
impl SendActivity for BanFromBoard {
  type Response = BanFromBoardResponse;

  async fn send_activity(
    request: &Self,
    _response: &Self::Response,
    context: &Data<TinyBoardsContext>,
    auth: Option<&str>,
  ) -> Result<(), TinyBoardsError> {
    let local_user_view = require_user(context.pool(), context.master_key(), auth).await.unwrap()?;
    let board: ApubBoard = Board::read(context.pool(), request.board_id)
      .await?
      .into();
    let banned_person: ApubPerson = Person::read(context.pool(), request.person_id)
      .await?
      .into();
    let expires = request.expires.map(naive_from_unix);

    if request.ban {
      BlockUser::send(
        &SiteOrBoard::Board(board),
        &banned_person,
        &local_user_view.person.clone().into(),
        request.remove_data.unwrap_or(false),
        request.reason.clone(),
        expires,
        context,
      )
      .await?;
    } else {
      UndoBlockUser::send(
        &SiteOrBoard::Board(board),
        &banned_person,
        &local_user_view.person.clone().into(),
        request.reason.clone(),
        context,
      )
      .await?;
    }
    Ok(())
  }
}