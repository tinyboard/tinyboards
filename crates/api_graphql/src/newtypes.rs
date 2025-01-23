/*
 * These are newtypes created only to be distinct types for various DataLoader implementations.
*/

// This macro is comfy and eliminates repetition
macro_rules! generate_newtypes {
    ( $($i:ident),+ ) => {
            $(
                #[derive(Clone, Hash, Eq, PartialEq)]
                pub(crate) struct $i(pub i32);

                impl From<i32> for $i {
                    fn from(value: i32) -> Self {
                        Self(value)
                    }
                }

                impl Into<i32> for $i {
                    fn into(self) -> i32 {
                        self.0
                    }
                }
            )+
        }
}

generate_newtypes![
    PersonId,
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
