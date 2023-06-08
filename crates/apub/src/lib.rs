use serde::Serialize;
use tinyboards_federation::config::Data;
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_utils::TinyBoardsError;
use url::Url;
use tinyboards_db::{
    models::apub::{activity::*, local_site::LocalSite, instance::Instance},
    traits::Crud,
};

pub mod api;
pub mod objects;


pub const FEDERATION_HTTP_FETCH_LIMIT: u32 = 50;

#[derive(Clone)]
pub(crate) struct LocalSiteData {
  local_site: Option<LocalSite>,
  allowed_instances: Vec<Instance>,
  blocked_instances: Vec<Instance>,
}

// pub(crate) fn check_ap_id_valid_with_strictness(
//     ap_id: &Url,
//     is_strict: bool,

// )

/// Store an activity in the database (sent or received).
///
/// Stored activities are served over the HTTP endpoint `GET /activities/{type_}/{id}`. This also
/// ensures that the same activity cannot be received more than once.
#[tracing::instrument(skip(data, activity))]
pub async fn insert_activity<T>(
    ap_id: &Url,
    activity: &T,
    local: bool,
    sensitive: bool,
    data: &Data<TinyBoardsContext>,
) -> Result<(), TinyBoardsError> 
where 
    T: Serialize
{
    let ap_id = Some(ap_id.clone().to_string());
    let activity_form = ActivityForm {
        ap_id,
        data: Some(serde_json::to_value(activity)?),
        local: Some(local),
        sensitive: Some(sensitive),
        updated: None,
    };

    Activity::create(data.pool(), &activity_form).await?;
    Ok(())
}

#[async_trait::async_trait]
pub trait SendActivity: Sync {
    type Response: Sync + Send;

    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
    ) -> Result<(), TinyBoardsError>;
}