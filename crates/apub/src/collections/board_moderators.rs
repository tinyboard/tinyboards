use crate::{
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::collections::group_moderators::GroupModerators,
};
use tinyboards_api_common::{data::TinyBoardsContext, utils::generate_moderators_url};
use tinyboards_db::{models::board::board_mods::*, traits::Joinable};
use tinyboards_db_views::structs::BoardModeratorView;
use tinyboards_federation::{
    config::Data, fetch::object_id::ObjectId, kinds::collection::OrderedCollectionType,
    protocol::verification::verify_domains_match, traits::Collection,
};
use tinyboards_utils::error::TinyBoardsError;
use url::Url;

#[derive(Clone, Debug)]
pub(crate) struct ApubBoardModerators(pub(crate) Vec<BoardModeratorView>);

#[async_trait::async_trait]
impl Collection for ApubBoardModerators {
    type Owner = ApubBoard;
    type DataType = TinyBoardsContext;
    type Kind = GroupModerators;
    type Error = TinyBoardsError;

    #[tracing::instrument(skip_all)]
    async fn read_local(
        owner: &Self::Owner,
        data: &Data<Self::DataType>,
    ) -> Result<Self::Kind, TinyBoardsError> {
        let moderators = BoardModeratorView::for_board(data.pool(), owner.id).await?;
        let ordered_items = moderators
            .into_iter()
            .map(|m| ObjectId::<ApubPerson>::from(m.moderator.actor_id))
            .collect();
        Ok(GroupModerators {
            r#type: OrderedCollectionType::OrderedCollection,
            id: generate_moderators_url(&owner.actor_id)?.into(),
            ordered_items,
        })
    }

    #[tracing::instrument(skip_all)]
    async fn verify(
        group_moderators: &GroupModerators,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), TinyBoardsError> {
        verify_domains_match(&group_moderators.id, expected_domain)?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn from_json(
        apub: Self::Kind,
        owner: &Self::Owner,
        data: &Data<Self::DataType>,
    ) -> Result<Self, TinyBoardsError> {
        let community_id = owner.id;
        let current_moderators = BoardModeratorView::for_board(data.pool(), community_id).await?;
        // Remove old mods from database which arent in the moderators collection anymore
        for mod_user in &current_moderators {
            let mod_id = ObjectId::from(mod_user.moderator.actor_id.clone());
            if !apub.ordered_items.contains(&mod_id) {
                let board_moderator_form = BoardModeratorForm {
                    board_id: Some(mod_user.board.id),
                    person_id: Some(mod_user.moderator.id),
                    ..BoardModeratorForm::default()
                };
                BoardModerator::leave(data.pool(), &board_moderator_form).await?;
            }
        }

        // Add new mods to database which have been added to moderators collection
        for mod_id in apub.ordered_items {
            let mod_user: ApubPerson = mod_id.dereference(data).await?;

            if !current_moderators
                .iter()
                .map(|c| c.moderator.actor_id.clone())
                .any(|x| x == mod_user.actor_id)
            {
                let board_moderator_form = BoardModeratorForm {
                    board_id: Some(owner.id),
                    person_id: Some(mod_user.id),
                    rank: Some(1),
                    invite_accepted: Some(true),
                    permissions: Some(ModPerms::Full.as_i32()),
                };
                BoardModerator::join(data.pool(), &board_moderator_form).await?;
            }
        }

        // This return value is unused, so just set an empty vec
        Ok(ApubBoardModerators(Vec::new()))
    }
}
