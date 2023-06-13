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

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SortType {
    Active,
    Hot,
    New,
    Old,
    TopDay,
    TopWeek,
    TopMonth,
    TopYear,
    TopAll,
    MostComments,
    NewComments,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum UserSortType {
    New,
    Old,
    MostRep,
    MostPosts,
    MostComments,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum CommentSortType {
    Hot,
    Top,
    New,
    Old,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ListingType {
    All,
    Subscribed,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SearchType {
    Person,
    Post,
    Comment,
    Board,
}

#[derive(EnumString, Display, Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum SubscribedType {
    Subscribed,
    NotSubscribed,
    Pending,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ModLogActionType {
    All,
    ModRemovePost,
    ModLockPost,
    ModStickyPost,
    ModRemoveComment,
    ModRemoveBoard,
    ModBanFromBoard,
    ModAddBoardMod,
    ModAddAdmin,
    ModBan,
    AdminPurgeUser,
    AdminPurgeBoard,
    AdminPurgePost,
    AdminPurgeComment,
}

pub fn map_to_modlog_type(match_string: Option<&str>) -> ModLogActionType {
    match match_string {
        Some("all") => ModLogActionType::All,
        Some("modremovepost") => ModLogActionType::ModRemovePost,
        Some("modlockpost") => ModLogActionType::ModLockPost,
        Some("modstickypost") => ModLogActionType::ModStickyPost,
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
