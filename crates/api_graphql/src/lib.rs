pub mod queries;
pub(crate) mod structs;
use async_graphql::*;
use queries::{me::MeQuery, posts::QueryPosts};
//use queries::Query;
use tinyboards_db_views::structs::LocalUserView;
use tinyboards_utils::TinyBoardsError;
//use tinyboards_api_common::data::TinyBoardsContext;

// wrapper around logged in user
pub struct LoggedInUser(Option<LocalUserView>);

#[derive(Default)]
pub struct TestQuery;

#[Object]
impl TestQuery {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

#[derive(MergedObject, Default)]
pub struct Query(TestQuery, MeQuery, QueryPosts);

pub fn gen_schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
    Schema::new(Query::default(), EmptyMutation, EmptySubscription)
}

impl From<Option<LocalUserView>> for LoggedInUser {
    fn from(value: Option<LocalUserView>) -> Self {
        Self(value)
    }
}

impl LoggedInUser {
    pub(crate) fn inner(&self) -> Option<&LocalUserView> {
        self.0.as_ref()
    }

    pub(crate) fn require_user(&self) -> Result<&LocalUserView> {
        match self.inner() {
            Some(v) => Ok(v),
            None => Err(TinyBoardsError::from_message(401, "Login required").into()),
        }
    }
}
