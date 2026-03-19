use async_graphql::*;

// Import all moderation query modules
pub use super::moderation::moderation_queue::ModerationQueueQueries;
pub use super::moderation::moderation_log::ModerationLogQueries;
pub use super::moderation::moderation_stats::ModerationStatsQueries;

#[derive(MergedObject, Default)]
pub struct ModerationQueries(
    ModerationQueueQueries,
    ModerationLogQueries,
    ModerationStatsQueries,
);