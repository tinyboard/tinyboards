pub mod context;
pub(crate) mod helpers;
pub(crate) mod loaders;
pub mod mutations;
pub(crate) mod newtypes;
pub mod queries;
pub(crate) mod structs;
pub mod utils;

use crate::mutations::{
    admin::{board_moderation::AdminBoardModeration, registration_applications::RegistrationApplicationMutations},
    auth::Auth,
    board::{actions::BoardActions, create::CreateBoard, settings::UpdateBoardSettings},
    board_moderation::BoardModerationMutations,
    message::{actions::MessageActionMutations, send_message::SendMessageMutations, edit_message::EditMessageMutations},
    notifications::NotificationMutations,
    user::{actions::UserActions, profile_management::ProfileManagement, settings::UpdateSettings},
    comment::{
        actions::*, edit::EditComment, moderation::CommentModeration, submit_comment::SubmitComment,
    },
    post::{actions::*, edit::EditPost, moderation::PostModeration, submit_post::SubmitPost},
    reports::ReportMutations,
    site::{config::SiteConfig, invite::SiteInvite},
};
use async_graphql::*;
use queries::{
    banned_users::QueryBannedUsers,
    board_moderators::QueryBoardModerators,
    boards::QueryBoards,
    invites::QueryInvites,
    site::QuerySite,
    me::MeQuery,
    messages::QueryMessages,
    notifications::QueryNotifications,
    user::QueryUser,
    posts::QueryPosts,
    registration_applications::RegistrationApplicationQueries,
    search::QuerySearch,
};
use tinyboards_db::{models::user::user::User, utils::DbPool};
//use queries::Query;
use tinyboards_utils::{settings::structs::Settings as Settings_, TinyBoardsError};
// Context moved to crate::context::TinyBoardsContext

/// wrapper around logged in user
pub struct LoggedInUser(Option<User>);
/// key for decoding JWTs
pub struct MasterKey(String);
/// Instance settings
pub struct Settings(&'static Settings_);

/// Dataloader for batch loading
pub struct PostgresLoader {
    pool: DbPool,
    // id of the logged in user to use in queries
    my_user_id: i32,
}

impl PostgresLoader {
    pub fn new(pool: &DbPool, my_user_id: i32) -> Self {
        Self {
            pool: pool.clone(),
            my_user_id,
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
    //TestQuery,
    MeQuery,
    QueryPosts,
    QueryBoards,
    QueryUser,
    QuerySite,
    QueryMessages,
    QueryNotifications,
    QueryInvites,
    QueryBoardModerators,
    QueryBannedUsers,
    QuerySearch,
    RegistrationApplicationQueries,
);

#[derive(MergedObject, Default)]
pub struct Mutation(
    AdminBoardModeration,
    Auth,
    BoardActions,
    CreateBoard,
    UpdateBoardSettings,
    UserActions,
    ProfileManagement,
    UpdateSettings,
    SubmitPost,
    SubmitComment,
    EditPost,
    PostActions,
    PostModeration,
    EditComment,
    CommentActions,
    CommentModeration,
    SendMessageMutations,
    EditMessageMutations,
    MessageActionMutations,
    SiteConfig,
    SiteInvite,
    NotificationMutations,
    ReportMutations,
    BoardModerationMutations,
    RegistrationApplicationMutations,
);

pub fn gen_schema() -> Schema<Query, Mutation, EmptySubscription> {
    Schema::new(Query::default(), Mutation::default(), EmptySubscription)
}

impl From<Option<User>> for LoggedInUser {
    fn from(value: Option<User>) -> Self {
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
    pub(crate) fn inner(&self) -> Option<&User> {
        self.0.as_ref()
    }

    pub(crate) fn require_user(&self) -> Result<&User> {
        match self.inner() {
            Some(v) => Ok(v),
            None => Err(TinyBoardsError::from_message(401, "Login required").into()),
        }
    }

    pub(crate) fn require_user_not_banned(&self) -> Result<&User> {
        match self.inner() {
            Some(v) => {
                if v.is_banned {
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
    fn censor(&mut self, my_user_id: i32, is_admin: bool, is_mod: bool);
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
