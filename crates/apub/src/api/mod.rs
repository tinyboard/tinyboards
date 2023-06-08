use tinyboards_db::{ListingType, models::site::site::Site};
use tinyboards_federation::config::Data;
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_utils::TinyBoardsError;

#[async_trait::async_trait]
pub trait PerformApub {
    type Response: serde::ser::Serialize + Send;

    async fn perform(&self, context: &Data<TinyBoardsContext>) -> Result<Self::Response, TinyBoardsError>;
}


// returns default listing type
fn listing_type_with_default(
    type_: Option<ListingType>,
    _local_site: &Site,
    board_id: Option<i32>,
) -> Result<ListingType, TinyBoardsError> {
    let listing_type = if board_id.is_none() {
        type_.unwrap_or(ListingType::All)
    } else {
        ListingType::All
    };

    // TODO - add default post listing type to the site table at some point and refer to it instead of ListingType::All in the unwrap_or

    Ok(listing_type)
}