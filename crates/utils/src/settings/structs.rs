use doku::Document;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default)]
pub struct RateLimitConfig {
  /// Maximum number of messages created in interval
  #[default(180)]
  pub message: i32,
  /// Interval length for message limit, in seconds
  #[default(60)]
  pub message_per_second: i32,
  /// Maximum number of posts created in interval
  #[default(6)]
  pub post: i32,
  /// Interval length for post limit, in seconds
  #[default(600)]
  pub post_per_second: i32,
  /// Maximum number of registrations in interval
  #[default(3)]
  pub register: i32,
  /// Interval length for registration limit, in seconds
  #[default(3600)]
  pub register_per_second: i32,
  /// Maximum number of image uploads in interval
  #[default(6)]
  pub image: i32,
  /// Interval length for image uploads, in seconds
  #[default(3600)]
  pub image_per_second: i32,
  /// Maximum number of comments created in interval
  #[default(6)]
  pub comment: i32,
  /// Interval length for comment limit, in seconds
  #[default(600)]
  pub comment_per_second: i32,
  #[default(60)]
  pub search: i32,
  /// Interval length for search limit, in seconds
  #[default(600)]
  pub search_per_second: i32,
}