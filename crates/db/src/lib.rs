#![recursion_limit = "256"]

pub mod aggregates;
pub mod database;
pub mod models;
pub mod newtypes;
pub mod traits;

pub mod impls;
pub mod schema;
pub mod utils;

pub use database::Database;


use serde::{Serialize, Deserialize};
use strum_macros::{Display, EnumString};

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
  All,
  Users,
  Posts,
  Comments,
  Boards,
}

#[derive(EnumString, Display, Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum SubscribedType {
  Subscribed,
  NotSubscribed,
  Pending,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ModlogActionType {
  All,
  ModRemovePost,
  ModLockPost,
  ModStickyPost,
  ModRemoveComment,
  ModRemoveCommunity,
  ModBanFromCommunity,
  ModAddCommunity,
  ModTransferCommunity,
  ModAdd,
  ModBan,
  ModHideCommunity,
  AdminPurgePerson,
  AdminPurgeCommunity,
  AdminPurgePost,
  AdminPurgeComment,
}

pub fn map_to_search_type(match_string: Option<&str>) -> SearchType {
    match match_string {
      Some("all") => SearchType::All,
      Some("users") => SearchType::Users,
      Some("posts") => SearchType::Posts,
      Some("comments") => SearchType::Comments,
      Some("boards") => SearchType::Boards,
      Some(&_) => SearchType::All,
      None => SearchType::All,
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
      None => SortType::Hot
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
      Some("Hot") => CommentSortType::Hot,
      Some("Top") => CommentSortType::Top,
      Some("New") => CommentSortType::New,
      Some("Old") => CommentSortType::Old,
      Some(&_) => CommentSortType::Hot,
      None => CommentSortType::Hot,
    }
}

pub fn map_to_listing_type(match_string: Option<&str>) -> ListingType {
    match match_string {
      Some("All") => ListingType::All,
      Some("Subscribed") => ListingType::Subscribed,
      Some(&_) => ListingType::All,
      None => ListingType::All,
    }
}