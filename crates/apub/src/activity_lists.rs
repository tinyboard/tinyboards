use crate::{
    objects::board::ApubBoard,
    protocol::{
        activities::{
            block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
            board::{
                announce::{AnnounceActivity, RawAnnouncableActivities},
                collection_add::CollectionAdd,
                collection_remove::CollectionRemove,
                lock_page::{LockPage, UndoLockPage},
                report::Report,
                update::UpdateBoard,
            },
            create_or_update::{note::CreateOrUpdateNote, page::CreateOrUpdatePage},
            deletion::{delete::Delete, delete_user::DeleteUser, undo_delete::UndoDelete},
            subscribed::{
                accept::AcceptSubscribe, subscribe::Subscribe, undo_subscribe::UndoSubscribe,
            },
            voting::{undo_vote::UndoVote, vote::Vote},
        },
        objects::page::Page,
        InBoard,
    },
};
use serde::{Deserialize, Serialize};
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_federation::{
    config::Data, protocol::context::WithContext, traits::ActivityHandler,
};
use tinyboards_utils::error::TinyBoardsError;
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum SharedInboxActivities {
    PersonInboxActivities(Box<WithContext<PersonInboxActivities>>),
    GroupInboxActivities(Box<WithContext<GroupInboxActivities>>),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum GroupInboxActivities {
    Subscribe(Subscribe),
    UndoSubscribe(UndoSubscribe),
    Report(Report),
    // catch-all
    AnnouncableActivities(RawAnnouncableActivities),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum PersonInboxActivities {
    Subscribe(Subscribe),
    AcceptSubscribe(AcceptSubscribe),
    UndoSubscribe(UndoSubscribe),
    Delete(Delete),
    UndoDelete(UndoDelete),
    AnnounceActivity(AnnounceActivity),
}

/// This is necessary for user inbox, which can also receive some "announcable" activities,
/// eg a comment mention. This needs to be a separate enum so that announcables received in shared
/// inbox can fall through to be parsed as GroupInboxActivities::AnnouncableActivities.
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum PersonInboxActivitiesWithAnnouncable {
    PersonInboxActivities(Box<PersonInboxActivities>),
    AnnouncableActivities(Box<AnnouncableActivities>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum AnnouncableActivities {
    CreateOrUpdateComment(CreateOrUpdateNote),
    CreateOrUpdatePost(CreateOrUpdatePage),
    Vote(Vote),
    UndoVote(UndoVote),
    Delete(Delete),
    UndoDelete(UndoDelete),
    UpdateBoard(UpdateBoard),
    BlockUser(BlockUser),
    UndoBlockUser(UndoBlockUser),
    CollectionAdd(CollectionAdd),
    CollectionRemove(CollectionRemove),
    LockPost(LockPage),
    UndoLockPost(UndoLockPage),
    // only send, no receive
    Page(Page),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
#[allow(clippy::enum_variant_names)]
pub enum SiteInboxActivities {
    BlockUser(BlockUser),
    UndoBlockUser(UndoBlockUser),
    DeleteUser(DeleteUser),
}

#[async_trait::async_trait]
impl InBoard for AnnouncableActivities {
    #[tracing::instrument(skip(self, context))]
    async fn board(&self, context: &Data<TinyBoardsContext>) -> Result<ApubBoard, TinyBoardsError> {
        use AnnouncableActivities::*;
        match self {
            CreateOrUpdateComment(a) => a.board(context).await,
            CreateOrUpdatePost(a) => a.board(context).await,
            Vote(a) => a.board(context).await,
            UndoVote(a) => a.board(context).await,
            Delete(a) => a.board(context).await,
            UndoDelete(a) => a.board(context).await,
            UpdateBoard(a) => a.board(context).await,
            BlockUser(a) => a.board(context).await,
            UndoBlockUser(a) => a.board(context).await,
            CollectionAdd(a) => a.board(context).await,
            CollectionRemove(a) => a.board(context).await,
            LockPost(a) => a.board(context).await,
            UndoLockPost(a) => a.board(context).await,
            Page(_) => unimplemented!(),
        }
    }
}
