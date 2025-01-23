use serde::{Deserialize, Serialize};
use strum_macros::Display;

pub mod block;
pub mod board;
pub mod create_or_update;
pub mod deletion;
pub mod subscribed;
pub mod voting;

#[derive(Clone, Debug, Display, Deserialize, Serialize, PartialEq, Eq)]
pub enum CreateOrUpdateType {
  Create,
  Update,
}