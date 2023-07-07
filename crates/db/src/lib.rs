#![recursion_limit = "512"]

pub mod aggregates;
pub mod database;
pub mod models;
pub mod newtypes;
pub mod traits;

pub mod impls;
pub mod schema;
pub mod utils;

pub use database::Database;

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum SiteMode {
    OpenMode,
    ApplicationMode,
    InviteMode,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationMode {
    /// Closed to the public
    #[strum(ascii_case_insensitive)]
    Closed,
    /// Open, but you need to have an approved application,
    #[strum(ascii_case_insensitive)]
    RequireApplication,
    /// Open, but an invite link is required
    #[strum(ascii_case_insensitive)]
    RequireInvite,
    /// Open to all
    #[strum(ascii_case_insensitive)]
    Open,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SortType {
    #[strum(ascii_case_insensitive)]
    Active,
    #[strum(ascii_case_insensitive)]
    Hot,
    #[strum(ascii_case_insensitive)]
    New,
    #[strum(ascii_case_insensitive)]
    Old,
    #[strum(ascii_case_insensitive)]
    TopDay,
    #[strum(ascii_case_insensitive)]
    TopWeek,
    #[strum(ascii_case_insensitive)]
    TopMonth,
    #[strum(ascii_case_insensitive)]
    TopYear,
    #[strum(ascii_case_insensitive)]
    TopAll,
    #[strum(ascii_case_insensitive)]
    MostComments,
    #[strum(ascii_case_insensitive)]
    NewComments,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum UserSortType {
    #[strum(ascii_case_insensitive)]
    New,
    #[strum(ascii_case_insensitive)]
    Old,
    #[strum(ascii_case_insensitive)]
    MostRep,
    #[strum(ascii_case_insensitive)]
    MostPosts,
    #[strum(ascii_case_insensitive)]
    MostComments,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum CommentSortType {
    #[strum(ascii_case_insensitive)]
    Hot,
    #[strum(ascii_case_insensitive)]
    Top,
    #[strum(ascii_case_insensitive)]
    New,
    #[strum(ascii_case_insensitive)]
    Old,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ListingType {
    #[strum(ascii_case_insensitive)]
    All,
    #[strum(ascii_case_insensitive)]
    Subscribed,
    #[strum(ascii_case_insensitive)]
    Local,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SearchType {
    #[strum(ascii_case_insensitive)]
    Person,
    #[strum(ascii_case_insensitive)]
    Post,
    #[strum(ascii_case_insensitive)]
    Comment,
    #[strum(ascii_case_insensitive)]
    Board,
}

#[derive(EnumString, Display, Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum SubscribedType {
    #[strum(ascii_case_insensitive)]
    Subscribed,
    #[strum(ascii_case_insensitive)]
    NotSubscribed,
    #[strum(ascii_case_insensitive)]
    Pending,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ModLogActionType {
    #[strum(ascii_case_insensitive)]
    All,
    #[strum(ascii_case_insensitive)]
    ModRemovePost,
    #[strum(ascii_case_insensitive)]
    ModLockPost,
    #[strum(ascii_case_insensitive)]
    ModFeaturePost,
    #[strum(ascii_case_insensitive)]
    ModRemoveComment,
    #[strum(ascii_case_insensitive)]
    ModRemoveBoard,
    #[strum(ascii_case_insensitive)]
    ModBanFromBoard,
    #[strum(ascii_case_insensitive)]
    ModAddBoardMod,
    #[strum(ascii_case_insensitive)]
    ModAddAdmin,
    #[strum(ascii_case_insensitive)]
    ModBan,
    #[strum(ascii_case_insensitive)]
    AdminPurgeUser,
    #[strum(ascii_case_insensitive)]
    AdminPurgeBoard,
    #[strum(ascii_case_insensitive)]
    AdminPurgePost,
    #[strum(ascii_case_insensitive)]
    AdminPurgeComment,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum PostFeatureType {
    /// Features to the top of the local site
    #[strum(ascii_case_insensitive)]
    Local,
    /// Features to the top of the board
    #[strum(ascii_case_insensitive)]
    Board,
}

pub fn map_to_modlog_type(match_string: Option<&str>) -> ModLogActionType {
    match match_string {
        Some("all") => ModLogActionType::All,
        Some("modremovepost") => ModLogActionType::ModRemovePost,
        Some("modlockpost") => ModLogActionType::ModLockPost,
        Some("modfeaturepost") => ModLogActionType::ModFeaturePost,
        Some("modremovecomment") => ModLogActionType::ModRemoveComment,
        Some("modremoveboard") => ModLogActionType::ModRemoveBoard,
        Some("modbanfromboard") => ModLogActionType::ModBanFromBoard,
        Some("modaddboardmod") => ModLogActionType::ModAddBoardMod,
        Some("modaddadmin") => ModLogActionType::ModAddAdmin,
        Some("modban") => ModLogActionType::ModBan,
        Some("adminpurgeboard") => ModLogActionType::AdminPurgeBoard,
        Some("adminpurgecomment") => ModLogActionType::AdminPurgeComment,
        Some("adminpurgepost") => ModLogActionType::AdminPurgePost,
        Some("adminpurgeuser") => ModLogActionType::AdminPurgeUser,
        Some(&_) => ModLogActionType::All,
        None => ModLogActionType::All,
    }
}

pub fn map_to_search_type(match_string: Option<&str>) -> SearchType {
    match match_string {
        Some("person") => SearchType::Person,
        Some("post") => SearchType::Post,
        Some("comment") => SearchType::Comment,
        Some("board") => SearchType::Board,
        Some(&_) => SearchType::Post,
        None => SearchType::Post,
    }
}

pub fn map_to_sort_type(match_string: Option<&str>) -> SortType {
    match match_string {
        Some("active") => SortType::Active,
        Some("hot") => SortType::Hot,
        Some("new") => SortType::New,
        Some("old") => SortType::Old,
        Some("topday") => SortType::TopDay,
        Some("topweek") => SortType::TopWeek,
        Some("topmonth") => SortType::TopMonth,
        Some("topall") => SortType::TopAll,
        Some("mostcomments") => SortType::MostComments,
        Some("newcomments") => SortType::NewComments,
        Some(&_) => SortType::Hot,
        None => SortType::Hot,
    }
}

pub fn map_to_user_sort_type(match_string: Option<&str>) -> UserSortType {
    match match_string {
        Some("new") => UserSortType::New,
        Some("old") => UserSortType::Old,
        Some("mostrep") => UserSortType::MostRep,
        Some("mostposts") => UserSortType::MostPosts,
        Some("mostcomments") => UserSortType::MostComments,
        Some(&_) => UserSortType::MostRep,
        None => UserSortType::MostRep,
    }
}

pub fn map_to_comment_sort_type(match_string: Option<&str>) -> CommentSortType {
    match match_string {
        Some("hot") => CommentSortType::Hot,
        Some("top") => CommentSortType::Top,
        Some("new") => CommentSortType::New,
        Some("old") => CommentSortType::Old,
        Some(&_) => CommentSortType::Hot,
        None => CommentSortType::Hot,
    }
}

pub fn map_to_listing_type(match_string: Option<&str>) -> ListingType {
    match match_string {
        Some("all") => ListingType::All,
        Some("subscribed") => ListingType::Subscribed,
        Some(&_) => ListingType::All,
        None => ListingType::All,
    }
}

pub fn map_to_post_feature_type(match_string: Option<&str>) -> PostFeatureType {
    match match_string {
        Some("local") => PostFeatureType::Local,
        Some("board") => PostFeatureType::Board,
        Some(&_) => PostFeatureType::Board,
        None => PostFeatureType::Board,
    }
}
