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
          update::UpdateCommunity,
        },
        create_or_update::{
          chat_message::CreateOrUpdateChatMessage,
          note::CreateOrUpdateNote,
          page::CreateOrUpdatePage,
        },
        deletion::{delete::Delete, delete_user::DeleteUser, undo_delete::UndoDelete},
        subscribed::{accept::AcceptFollow, follow::Follow, undo_follow::UndoFollow},
        voting::{undo_vote::UndoVote, vote::Vote},
      },
      objects::page::Page,
      InBoard,
    },
  };
  use tinyboards_federation::{
    config::Data,
    protocol::context::WithContext,
    traits::ActivityHandler,
  };
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_utils::error::TinyBoardsError;
  use serde::{Deserialize, Serialize};
  use url::Url;



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
}