#![recursion_limit = "256"]

pub mod aggregates;
pub mod database;
pub mod models;
pub mod porpl_types;
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
  Comments,
  Posts,
  Communities,
  Users,
  Url,
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

pub fn map_to_sort_type(match_string: Option<&str>) -> SortType {
    match match_string {
      Some("Active") => SortType::Active,
      Some("Hot") => SortType::Hot,
      Some("New") => SortType::New,
      Some("Old") => SortType::Old,
      Some("TopDay") => SortType::TopDay,
      Some("TopWeek") => SortType::TopWeek,
      Some("TopMonth") => SortType::TopMonth,
      Some("TopAll") => SortType::TopAll,
      Some("MostComments") => SortType::MostComments,
      Some("NewComments") => SortType::NewComments,
      Some(&_) => SortType::Hot,
      None => SortType::Hot
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