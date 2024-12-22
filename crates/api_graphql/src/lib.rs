pub(crate) mod helpers;
pub(crate) mod loaders;
pub mod mutations;
pub(crate) mod newtypes;
pub mod queries;
pub(crate) mod structs;

use crate::mutations::{
    auth::Auth,
    comment::{
        actions::*, edit::EditComment, moderation::CommentModeration, submit_comment::SubmitComment,
    },
    post::{actions::*, edit::EditPost, moderation::PostModeration, submit_post::SubmitPost},
};
use async_graphql::*;
use queries::{
    boards::QueryBoards, local_site::QuerySite, me::MeQuery, person::QueryPerson, posts::QueryPosts,
};
use tinyboards_db::utils::DbPool;
//use queries::Query;
use tinyboards_db_views::structs::LocalUserView;
use tinyboards_utils::{settings::structs::Settings as Settings_, TinyBoardsError};
//use tinyboards_api_common::data::TinyBoardsContext;

/// wrapper around logged in user
pub struct LoggedInUser(Option<LocalUserView>);
/// key for decoding JWTs
pub struct MasterKey(String);
/// Instance settings
pub struct Settings(&'static Settings_);

/// Dataloader for batch loading
pub struct PostgresLoader {
    pool: DbPool,
    // id of the logged in person to use in queries
    my_person_id: i32,
}

impl PostgresLoader {
    pub fn new(pool: &DbPool, my_person_id: i32) -> Self {
        Self {
            pool: pool.clone(),
            my_person_id,
        }
    }
}

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
pub struct Query(
    TestQuery,
    MeQuery,
    QueryPosts,
    QueryBoards,
    QueryPerson,
    QuerySite,
);

#[derive(MergedObject, Default)]
pub struct Mutation(
    Auth,
    SubmitPost,
    SubmitComment,
    EditPost,
    PostActions,
    PostModeration,
    EditComment,
    CommentActions,
    CommentModeration,
);

pub fn gen_schema() -> Schema<Query, Mutation, EmptySubscription> {
    Schema::new(Query::default(), Mutation::default(), EmptySubscription)
}

impl From<Option<LocalUserView>> for LoggedInUser {
    fn from(value: Option<LocalUserView>) -> Self {
        Self(value)
    }
}

impl From<String> for MasterKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&'static Settings_> for Settings {
    fn from(value: &'static Settings_) -> Self {
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

    pub(crate) fn require_user_not_banned(&self) -> Result<&LocalUserView> {
        match self.inner() {
            Some(v) => {
                if v.person.is_banned {
                    Err(TinyBoardsError::from_message(403, "Your account is banned").into())
                } else {
                    Ok(v)
                }
            }
            None => Err(TinyBoardsError::from_message(401, "Login required").into()),
        }
    }
}

impl MasterKey {
    pub(crate) fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Settings {
    pub(crate) fn as_ref(&self) -> &'static Settings_ {
        self.0
    }
}

// censor contents of deleted/removed posts/comments
pub(crate) trait Censorable {
    fn censor(&mut self, my_person_id: i32, is_admin: bool, is_mod: bool);
}

// custom enums from the db crate
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(remote = "tinyboards_db::SortType")]
pub enum SortType {
    #[graphql(name = "active")]
    Active,
    #[graphql(name = "hot")]
    Hot,
    #[graphql(name = "new")]
    New,
    #[graphql(name = "old")]
    Old,
    #[graphql(name = "topDay")]
    TopDay,
    #[graphql(name = "topWeek")]
    TopWeek,
    #[graphql(name = "topMonth")]
    TopMonth,
    #[graphql(name = "topYear")]
    TopYear,
    #[graphql(name = "topAll")]
    TopAll,
    #[graphql(name = "mostComments")]
    MostComments,
    #[graphql(name = "newComments")]
    NewComments,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(remote = "tinyboards_db::ListingType")]
pub enum ListingType {
    #[graphql(name = "all")]
    All,
    #[graphql(name = "subscribed")]
    Subscribed,
    #[graphql(name = "local")]
    Local,
    #[graphql(name = "moderated")]
    Moderated,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "tinyboards_db::CommentSortType")]
pub enum CommentSortType {
    #[graphql(name = "hot")]
    Hot,
    #[graphql(name = "top")]
    Top,
    #[graphql(name = "new")]
    New,
    #[graphql(name = "old")]
    Old,
}

#[derive(Enum, Clone, Copy, Eq, PartialEq)]
#[graphql(remote = "tinyboards_db::UserSortType")]
pub enum UserSortType {
    #[graphql(name = "new")]
    New,
    #[graphql(name = "old")]
    Old,
    #[graphql(name = "mostRep")]
    MostRep,
    #[graphql(name = "mostPosts")]
    MostPosts,
    #[graphql(name = "mostComments")]
    MostComments,
}

#[derive(Enum, Clone, Copy, Eq, PartialEq)]
#[graphql(remote = "tinyboards_db::UserListingType")]
pub enum UserListingType {
    #[graphql(name = "all")]
    All,
    #[graphql(name = "banned")]
    Banned,
    #[graphql(name = "notBanned")]
    NotBanned,
    #[graphql(name = "admins")]
    Admins,
}

#[derive(Enum, Clone, Copy, Eq, PartialEq)]
#[graphql(remote = "tinyboards_db::SubscribedType")]
pub enum SubscribedType {
    #[graphql(name = "subscribed")]
    Subscribed,
    #[graphql(name = "notSubscribed")]
    NotSubscribed,
    #[graphql(name = "pending")]
    Pending,
}
