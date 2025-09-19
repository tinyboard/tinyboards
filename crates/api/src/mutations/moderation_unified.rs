use async_graphql::*;

// Import all moderation modules
pub use super::moderation::site_moderation::SiteModerationMutations;
pub use super::moderation::board_moderation::BoardBanMutations;
pub use super::moderation::report_moderation::ReportModerationMutations;

#[derive(MergedObject, Default)]
pub struct ModerationMutations(
    SiteModerationMutations,
    BoardBanMutations,
    ReportModerationMutations,
);