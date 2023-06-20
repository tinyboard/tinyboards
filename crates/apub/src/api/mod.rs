use tinyboards_db::{ListingType, models::site::{site::Site, local_site::LocalSite}, map_to_listing_type};
use tinyboards_federation::config::Data;
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_utils::TinyBoardsError;

pub mod list_comments;
pub mod list_posts;
pub mod read_board;
pub mod read_person;
pub mod resolve_object;
pub mod search;

#[async_trait::async_trait]
pub trait PerformApub {
    type Response: serde::ser::Serialize + Send;

    async fn perform(&self, context: &Data<TinyBoardsContext>, auth: Option<&str>) -> Result<Self::Response, TinyBoardsError>;
}


// returns default listing type
fn listing_type_with_default(
    type_: Option<ListingType>,
    local_site: &LocalSite,
    board_id: Option<i32>,
) -> Result<ListingType, TinyBoardsError> {
    let listing_type = if board_id.is_none() {
        type_.unwrap_or(map_to_listing_type(Some(local_site.default_post_listing_type.to_lowercase().as_str())))
    } else {
        // inside of board, show everything
        ListingType::All
    };
    
    Ok(listing_type)
}