/*
 * These are newtypes created only to be distinct types for various DataLoader implementations.
*/

#[derive(Clone, Hash, Eq, PartialEq)]
pub(crate) struct PersonId(i32);

impl From<i32> for PersonId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl Into<i32> for PersonId {
    fn into(self) -> i32 {
        self.0
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub(crate) struct BoardIdForPost(pub i32);

impl From<i32> for BoardIdForPost {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl Into<i32> for BoardIdForPost {
    fn into(self) -> i32 {
        self.0
    }
}
