use serde::Serialize;
use tinyboards_federation::config::{Data, UrlVerifier};
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_utils::{TinyBoardsError, settings::structs::Settings};
use url::Url;
use tinyboards_db::{
    models::apub::{activity::*, instance::Instance},
    models::site::local_site::LocalSite,
    traits::Crud, utils::DbPool,
};
use once_cell::sync::Lazy;

pub mod api;
pub mod objects;
pub mod protocol;
pub mod activities;
pub mod collections;
pub mod activity_lists;

pub const FEDERATION_HTTP_FETCH_LIMIT: u32 = 50;
static CONTEXT: Lazy<Vec<serde_json::Value>> = Lazy::new(|| {
    serde_json::from_str(include_str!("../assets/tinyboards/context.json")).expect("parse apub context")
});

#[derive(Clone)]
pub struct VerifyUrlData(pub DbPool);

#[async_trait::async_trait]
impl UrlVerifier for VerifyUrlData {
    async fn verify(&self, url: &Url) -> Result<(), &'static str> {

        let local_site_data = fetch_local_site_data(&self.0)
            .await
            .expect("read local site data");

        check_ap_id_valid(url, &local_site_data)?;

        Ok(())

    }
}


#[derive(Clone)]
pub(crate) struct LocalSiteData {
  local_site: LocalSite,
  allowed_instances: Vec<Instance>,
  blocked_instances: Vec<Instance>,
}

pub(crate) async fn fetch_local_site_data(pool: &DbPool) -> Result<LocalSiteData, diesel::result::Error> {
    let local_site = LocalSite::read(pool).await?;
    let allowed_instances = Instance::allow_list(pool).await?;
    let blocked_instances = Instance::block_list(pool).await?;
    Ok( LocalSiteData { local_site, allowed_instances, blocked_instances })
}

/// Checks if ap id is allowed for sending or receiving
/// 
/// In particular it will check for:
///     - federation being enabled
///     - the correct scheme (either http or https)
///     - URL being in the allow list (if active)
///     - URL not being in the block list (if active)
#[tracing::instrument(skip(local_site_data))]
fn check_ap_id_valid(
    ap_id: &Url,
    local_site_data: &LocalSiteData,
) -> Result<(), &'static str> {
    let domain = ap_id.domain().expect("ap id has domain.").to_string();

    if !local_site_data.local_site.federation_enabled {
        return Err("federation disabled.");
    }

    if local_site_data
        .blocked_instances
        .iter()
        .any(|i| domain.eq(&i.domain)) {

        return Err("domain is blocked.");
    }

    // only check this if instances are in the allow list
    if !local_site_data.allowed_instances.is_empty() &&
       !local_site_data
        .allowed_instances
        .iter()
        .any(|i| domain.eq(&i.domain)) {
            return Err("domain is not in allow list.");
    }

    Ok(())
}

pub(crate) fn check_ap_id_valid_with_strictness(
    ap_id: &Url,
    is_strict: bool,
    local_site_data: &LocalSiteData,
    settings: &Settings,
) -> Result<(), TinyBoardsError> {

    // only check the allow list if the ap_id is a board and allow list has some items in it
    if is_strict && !local_site_data.allowed_instances.is_empty() {
        // need to allow this explicitly because the apub receive might contain objects from the local instance
        let mut allowed_and_local = local_site_data
            .allowed_instances
            .iter()
            .map(|i| i.domain.clone())
            .collect::<Vec<String>>();
        let local_instance 
            = settings.get_hostname_without_port()?;
        allowed_and_local
            .push(local_instance);
        let domain 
            = ap_id.domain().expect("apub id has domain").to_string();
        
        if !allowed_and_local.contains(&domain) {
            return Err(TinyBoardsError::from_message(403, "federation forbidden by a strict allow list."));
        }      
    }
    Ok(())
}

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