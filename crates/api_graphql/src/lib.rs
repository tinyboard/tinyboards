pub mod queries;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use queries::Query;
use tinyboards_db_views::structs::LocalUserView;
//use tinyboards_api_common::data::TinyBoardsContext;

// wrapper around logged in user
pub struct LoggedInUser(Option<LocalUserView>);

pub fn gen_schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
    Schema::new(Query, EmptyMutation, EmptySubscription)
}

impl From<Option<LocalUserView>> for LoggedInUser {
    fn from(value: Option<LocalUserView>) -> Self {
        Self(value)
    }
}

impl LoggedInUser {
    pub(crate) fn into_inner(self) -> Option<LocalUserView> {
        self.0
    }

    pub(crate) fn inner(&self) -> Option<&LocalUserView> {
        self.0.as_ref()
    }

    pub(crate) fn require_user(self) -> LocalUserView {
        match self.into_inner() {
            Some(v) => v,
            None => todo!("this should be an error"),
        }
    }
}
