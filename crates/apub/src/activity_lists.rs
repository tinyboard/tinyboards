use serde::{Deserialize, Serialize};
use tinyboards_federation::traits::ActivityHandler;




#[derive(Clone, Debug, Deserialize, Serialize)]
// #[serde(untagged)]
// #[enum_delegate::implement(ActivityHandler)]
pub enum AnnouncableActivities {
    // CreateOrUpdateComment(CreateOrUpdateNote),
    // CreateOrUpdatePost(CreateOrUpdatePage),
    // Vote(Vote),
    // UndoVote(UndoVote),
    // Delete(Delete),
    // UndoDelete(UndoDelete),
    // UpdateBoard(UpdateBoard),
    // BlockUser(BlockUser),
    // UndoBlockUser(UndoBlockUser),
    // CollectionAdd(CollectionAdd),
    // CollectionRemove(CollectionRemove),
    // LockPost(LockPage),
    // UndoLockPost(UndoLockPage),
}