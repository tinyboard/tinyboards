/*
 * These are newtypes created only to be distinct types for various DataLoader implementations.
*/

use uuid::Uuid;

// This macro is comfy and eliminates repetition
macro_rules! generate_newtypes {
    ( $($i:ident),+ ) => {
            $(
                #[derive(Clone, Hash, Eq, PartialEq)]
                pub(crate) struct $i(pub Uuid);

                impl From<Uuid> for $i {
                    fn from(value: Uuid) -> Self {
                        Self(value)
                    }
                }

                impl Into<Uuid> for $i {
                    fn into(self) -> Uuid {
                        self.0
                    }
                }
            )+
        }
}

generate_newtypes![
    UserId,
    PostIdForComment,
    VoteForCommentId,
    SavedForCommentId,
    BoardId,
    VoteForPostId,
    SavedForPostId,
    ModPermsForPostId,
    ModPermsForBoardId,
    SubscribedTypeForBoardId
];
